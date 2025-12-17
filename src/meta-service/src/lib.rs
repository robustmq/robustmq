// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::result_large_err)]
use crate::controller::call_broker::mqtt::{broker_call_thread_manager, BrokerCallManager};
use crate::controller::connector::scheduler::start_connector_scheduler;
use crate::core::cache::{load_cache, CacheManager};
use crate::core::controller::ClusterController;
use crate::raft::manager::MultiRaftManager;
use grpc_clients::pool::ClientPool;
use raft::leadership::monitoring_leader_transition;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

pub mod controller;
pub mod core;
pub mod raft;
pub mod server;
pub mod storage;

#[derive(Clone)]
pub struct MetaServiceServerParams {
    pub raft_manager: Arc<MultiRaftManager>,
    // Cache metadata information for the Storage Engine cluster
    pub cache_manager: Arc<CacheManager>,
    // Raft Global read and write pointer
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    // Global GRPC client connection pool
    pub client_pool: Arc<ClientPool>,
    // Global call thread manager
    pub call_manager: Arc<BrokerCallManager>,
}
pub struct MetaServiceServer {
    raft_manager: Arc<MultiRaftManager>,
    // Cache metadata information for the Storage Engine cluster
    cache_manager: Arc<CacheManager>,
    // Raft Global read and write pointer
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    // Global GRPC client connection pool
    client_pool: Arc<ClientPool>,
    // Global call thread manager
    broker_call_manager: Arc<BrokerCallManager>,
    main_stop: broadcast::Sender<bool>,
    inner_stop: broadcast::Sender<bool>,
}

impl MetaServiceServer {
    pub fn new(
        params: MetaServiceServerParams,
        main_stop: broadcast::Sender<bool>,
    ) -> MetaServiceServer {
        let (inner_stop, _) = broadcast::channel(2);
        MetaServiceServer {
            cache_manager: params.cache_manager,
            rocksdb_engine_handler: params.rocksdb_engine_handler,
            client_pool: params.client_pool,
            broker_call_manager: params.call_manager,
            raft_manager: params.raft_manager,
            main_stop,
            inner_stop,
        }
    }

    pub async fn start(&mut self) {
        self.start_init();

        self.start_raft_machine().await;

        self.start_heartbeat();

        self.start_controller();

        self.awaiting_stop().await;
    }

    pub fn start_heartbeat(&self) {
        let ctrl = ClusterController::new(
            self.cache_manager.clone(),
            self.raft_manager.clone(),
            self.inner_stop.clone(),
            self.client_pool.clone(),
            self.broker_call_manager.clone(),
        );

        tokio::spawn(async move {
            ctrl.start_node_heartbeat_check().await;
        });
    }

    async fn start_raft_machine(&self) {
        // create raft node
        if let Err(e) = self.raft_manager.start().await {
            panic!("{}", e);
        }

        // monitor leader switch
        monitoring_leader_transition(
            self.rocksdb_engine_handler.clone(),
            self.cache_manager.clone(),
            self.client_pool.clone(),
            self.raft_manager.clone(),
            self.main_stop.clone(),
        );
    }

    fn start_controller(&self) {
        let broker_call_manager = self.broker_call_manager.clone();
        let client_pool = self.client_pool.clone();
        let stop_send = self.inner_stop.clone();
        tokio::spawn(async move {
            broker_call_thread_manager(&broker_call_manager, &client_pool, stop_send).await;
        });

        // start mqtt connector scheduler thread
        let call_manager = self.broker_call_manager.clone();
        let client_pool = self.client_pool.clone();
        let cache_manager = self.cache_manager.clone();
        let raft_manager = self.raft_manager.clone();
        let stop_send = self.inner_stop.clone();
        tokio::spawn(async move {
            start_connector_scheduler(
                &cache_manager,
                &raft_manager,
                &call_manager,
                &client_pool,
                stop_send,
            )
            .await;
        });
    }

    pub fn start_init(&self) {
        if let Err(e) = load_cache(&self.cache_manager, &self.rocksdb_engine_handler) {
            panic!("Failed to load Cache,{e}");
        }
    }

    pub async fn awaiting_stop(&self) {
        let main_stop = self.main_stop.clone();
        let raft_manager = self.raft_manager.clone();
        let inner_stop = self.inner_stop.clone();
        // Stop the Server first, indicating that it will no longer receive request packets.
        let mut recv = main_stop.subscribe();
        match recv.recv().await {
            Ok(_) => {
                info!("Meta service shutdown initiated...");

                // Step 1: Stop all background threads (GC, heartbeat, controllers)
                info!("Stopping background threads...");
                if let Err(e) = inner_stop.send(true) {
                    error!("Failed to send stop signal to background threads: {}", e);
                }

                // Step 2: Wait for background threads to finish gracefully
                info!("Waiting for background threads to complete...");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                // Step 3: Shutdown Raft node
                info!("Shutting down Raft node...");
                if let Err(e) = raft_manager.shutdown().await {
                    error!("Failed to shutdown Raft node: {}", e);
                } else {
                    info!("Raft node shutdown successfully.");
                }

                info!("Meta service stopped gracefully.");
            }
            Err(e) => {
                error!("Failed to receive stop signal: {}", e);
            }
        }
    }
}
