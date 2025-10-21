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
use crate::controller::journal::call_node::{journal_call_thread_manager, JournalInnerCallManager};
use crate::controller::mqtt::call_broker::{mqtt_call_thread_manager, MQTTInnerCallManager};
use crate::controller::mqtt::connector::scheduler::start_connector_scheduler;
use crate::core::cache::{load_cache, CacheManager};
use crate::core::controller::ClusterController;
use crate::raft::raft_node::start_raft_node;
use crate::raft::route::apply::StorageDriver;
use crate::raft::type_config::TypeConfig;
use grpc_clients::pool::ClientPool;
use openraft::Raft;
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
    pub raf_node: Raft<TypeConfig>,
    pub storage_driver: Arc<StorageDriver>,
    // Cache metadata information for the Storage Engine cluster
    pub cache_manager: Arc<CacheManager>,
    // Raft Global read and write pointer
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    // Global GRPC client connection pool
    pub client_pool: Arc<ClientPool>,
    // Global call thread manager
    pub journal_call_manager: Arc<JournalInnerCallManager>,
    // Global call thread manager
    pub mqtt_call_manager: Arc<MQTTInnerCallManager>,
}
pub struct MetaServiceServer {
    raf_node: Raft<TypeConfig>,
    storage_driver: Arc<StorageDriver>,
    // Cache metadata information for the Storage Engine cluster
    cache_manager: Arc<CacheManager>,
    // Raft Global read and write pointer
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    // Global GRPC client connection pool
    client_pool: Arc<ClientPool>,
    // Global call thread manager
    journal_call_manager: Arc<JournalInnerCallManager>,
    // Global call thread manager
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
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
            journal_call_manager: params.journal_call_manager,
            mqtt_call_manager: params.mqtt_call_manager,
            raf_node: params.raf_node,
            storage_driver: params.storage_driver,
            main_stop,
            inner_stop,
        }
    }

    pub async fn start(&mut self) {
        self.start_init();

        self.start_raft_machine();

        self.start_heartbeat();

        self.start_mqtt_controller();

        self.start_journal_controller();

        self.awaiting_stop().await;
    }

    pub fn start_heartbeat(&self) {
        let ctrl = ClusterController::new(
            self.cache_manager.clone(),
            self.storage_driver.clone(),
            self.inner_stop.clone(),
            self.client_pool.clone(),
            self.journal_call_manager.clone(),
            self.mqtt_call_manager.clone(),
        );

        tokio::spawn(async move {
            ctrl.start_node_heartbeat_check().await;
        });
    }

    fn start_raft_machine(&self) {
        // create raft node
        let raft_node = self.raf_node.clone();

        tokio::spawn(async move {
            start_raft_node(raft_node).await;
        });

        // monitor leader switch
        monitoring_leader_transition(
            &self.raf_node,
            self.rocksdb_engine_handler.clone(),
            self.cache_manager.clone(),
            self.client_pool.clone(),
            self.storage_driver.clone(),
            self.main_stop.clone(),
        );
    }

    fn start_mqtt_controller(&self) {
        let mqtt_all_manager = self.mqtt_call_manager.clone();
        let client_pool = self.client_pool.clone();
        let stop_send = self.inner_stop.clone();
        tokio::spawn(async move {
            mqtt_call_thread_manager(&mqtt_all_manager, &client_pool, stop_send).await;
        });

        // start mqtt connector scheduler thread
        let call_manager = self.mqtt_call_manager.clone();
        let client_pool = self.client_pool.clone();
        let cache_manager = self.cache_manager.clone();
        let raft_machine_apply = self.storage_driver.clone();
        let stop_send = self.inner_stop.clone();
        tokio::spawn(async move {
            start_connector_scheduler(
                &cache_manager,
                &raft_machine_apply,
                &call_manager,
                &client_pool,
                stop_send,
            )
            .await;
        });
    }

    fn start_journal_controller(&self) {
        let client_pool = self.client_pool.clone();
        let journal_all_manager = self.journal_call_manager.clone();
        let stop_send = self.inner_stop.clone();
        tokio::spawn(async move {
            journal_call_thread_manager(&journal_all_manager, &client_pool, stop_send).await;
        });
    }

    pub fn start_init(&self) {
        if let Err(e) = load_cache(&self.cache_manager, &self.rocksdb_engine_handler) {
            panic!("Failed to load Cache,{e}");
        }
    }

    pub async fn awaiting_stop(&self) {
        let main_stop = self.main_stop.clone();
        let raw_raf_node = self.raf_node.clone();
        let inner_stop = self.inner_stop.clone();
        // Stop the Server first, indicating that it will no longer receive request packets.
        let mut recv = main_stop.subscribe();
        match recv.recv().await {
            Ok(_) => {
                info!("Meta service has stopped.");
                if let Err(e) = raw_raf_node.shutdown().await {
                    error!("{}", e);
                }

                if let Err(e) = inner_stop.send(true) {
                    error!("{}", e);
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}
