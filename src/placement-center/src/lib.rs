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
use crate::raft::raft_node::{create_raft_node, start_raft_node};
use crate::raft::route::apply::StorageDriver;
use crate::raft::route::DataRoute;
use crate::raft::type_config::TypeConfig;
use common_base::metrics::register_prometheus_export;
use common_base::version::logo::banner;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use openraft::Raft;
use raft::leadership::monitoring_leader_transition;
use server::grpc_server::start_grpc_server;
use std::sync::Arc;
use std::time::Duration;
use storage::rocksdb::{column_family_list, storage_data_fold, RocksDBEngine};
use tokio::signal;
use tokio::sync::broadcast::Sender;
use tokio::time::sleep;
use tracing::info;

pub mod controller;
pub mod core;
mod raft;
mod server;
mod storage;

pub struct PlacementCenter {
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
}

impl PlacementCenter {
    pub async fn new() -> PlacementCenter {
        let config = broker_config();
        let client_pool = Arc::new(ClientPool::new(100));
        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));

        let cache_manager: Arc<CacheManager> =
            Arc::new(CacheManager::new(rocksdb_engine_handler.clone()));

        let journal_call_manager = Arc::new(JournalInnerCallManager::new(cache_manager.clone()));
        let mqtt_call_manager = Arc::new(MQTTInnerCallManager::new(cache_manager.clone()));

        let data_route = Arc::new(DataRoute::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
        ));
        let raf_node: Raft<TypeConfig> = create_raft_node(client_pool.clone(), data_route).await;
        let storage_driver: Arc<StorageDriver> = Arc::new(StorageDriver::new(raf_node.clone()));
        PlacementCenter {
            cache_manager,
            rocksdb_engine_handler,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
            raf_node,
            storage_driver,
        }
    }

    pub async fn start(&mut self, stop_send: Sender<bool>) {
        self.start_init();

        self.start_raft_machine();

        self.start_heartbeat(stop_send.clone());

        self.start_prometheus();

        self.start_mqtt_controller(stop_send.clone());

        self.start_journal_controller();

        self.start_grpc_server();

        self.awaiting_stop(stop_send).await;
    }

    pub fn start_grpc_server(&self) {
        let cache_manager = self.cache_manager.clone();
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let client_pool = self.client_pool.clone();
        let journal_call_manager = self.journal_call_manager.clone();
        let mqtt_call_manager = self.mqtt_call_manager.clone();
        let storage_driver = self.storage_driver.clone();
        tokio::spawn(async move {
            if let Err(e) = start_grpc_server(
                storage_driver,
                cache_manager,
                rocksdb_engine_handler,
                client_pool,
                journal_call_manager,
                mqtt_call_manager,
            )
            .await
            {
                panic!("Failed to start grpc server,{e}");
            }
        });
    }

    pub fn start_heartbeat(&self, stop_send: Sender<bool>) {
        let ctrl = ClusterController::new(
            self.cache_manager.clone(),
            self.storage_driver.clone(),
            stop_send.clone(),
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
        );
    }

    fn start_prometheus(&self) {
        let conf = broker_config();
        if conf.prometheus.enable {
            tokio::spawn(async move {
                register_prometheus_export(conf.prometheus.port).await;
            });
        }
    }

    fn start_mqtt_controller(&self, stop_send: Sender<bool>) {
        let mqtt_all_manager = self.mqtt_call_manager.clone();
        let client_pool = self.client_pool.clone();
        tokio::spawn(async move {
            mqtt_call_thread_manager(&mqtt_all_manager, &client_pool).await;
        });

        // start mqtt connector scheduler thread
        let call_manager = self.mqtt_call_manager.clone();
        let client_pool = self.client_pool.clone();
        let cache_manager = self.cache_manager.clone();
        let raft_machine_apply = self.storage_driver.clone();
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
        tokio::spawn(async move {
            journal_call_thread_manager(&journal_all_manager, &client_pool).await;
        });
    }

    // Wait Stop Signal
    pub async fn awaiting_stop(&self, stop_send: Sender<bool>) {
        tokio::spawn(async move {
            sleep(Duration::from_millis(5)).await;
            info!("Placement Center service started successfully...");
        });

        signal::ctrl_c().await.expect("failed to listen for event");
        if stop_send.send(true).is_ok() {
            info!("When ctrl + c is received, the service starts to stop");
        }
    }

    pub fn start_init(&self) {
        banner();
        if let Err(e) = load_cache(&self.cache_manager, &self.rocksdb_engine_handler) {
            panic!("Failed to load Cache,{e}");
        }
    }
}
