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
use std::sync::Arc;
use std::time::Duration;

use common_base::metrics::register_prometheus_export;
use common_config::place::config::placement_center_conf;
use grpc_clients::pool::ClientPool;
use mqtt::cache::load_mqtt_cache;
use mqtt::connector::scheduler::start_connector_scheduler;
use mqtt::controller::call_broker::{mqtt_call_thread_manager, MQTTInnerCallManager};
use openraft::Raft;
use raft::leadership::monitoring_leader_transition;
use server::grpc_server::start_grpc_server;
use storage::rocksdb::{column_family_list, storage_data_fold, RocksDBEngine};
use tokio::signal;
use tokio::sync::broadcast::Sender;
use tokio::time::sleep;
use tracing::info;

use crate::core::cache::PlacementCacheManager;
use crate::core::controller::ClusterController;
use crate::journal::cache::{load_journal_cache, JournalCacheManager};
use crate::journal::controller::call_node::{journal_call_thread_manager, JournalInnerCallManager};
use crate::mqtt::cache::MqttCacheManager;
use crate::raft::raft_node::{create_raft_node, start_openraft_node};
use crate::raft::route::apply::RaftMachineApply;
use crate::raft::route::DataRoute;
use crate::raft::typeconfig::TypeConfig;

pub mod core;
mod inner;
mod journal;
mod kv;
mod mqtt;
mod raft;
mod server;
mod storage;

pub struct PlacementCenter {
    // Cache metadata information for the Storage Engine cluster
    cluster_cache: Arc<PlacementCacheManager>,
    // Cache metadata information for the Broker Server cluster
    engine_cache: Arc<JournalCacheManager>,
    // Cache metadata information for the MQTT Server cluster
    mqtt_cache: Arc<MqttCacheManager>,
    // Raft Global read and write pointer
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    // Global GRPC client connection pool
    client_pool: Arc<ClientPool>,
    // Global call thread manager
    journal_call_manager: Arc<JournalInnerCallManager>,
    // Global call thread manager
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
}

impl Default for PlacementCenter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlacementCenter {
    pub fn new() -> PlacementCenter {
        let config = placement_center_conf();
        let client_pool = Arc::new(ClientPool::new(100));
        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));

        let engine_cache = Arc::new(JournalCacheManager::new());
        let cluster_cache: Arc<PlacementCacheManager> =
            Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let mqtt_cache: Arc<MqttCacheManager> = Arc::new(MqttCacheManager::new());

        let journal_call_manager = Arc::new(JournalInnerCallManager::new(cluster_cache.clone()));
        let mqtt_call_manager = Arc::new(MQTTInnerCallManager::new(cluster_cache.clone()));
        PlacementCenter {
            cluster_cache,
            engine_cache,
            mqtt_cache,
            rocksdb_engine_handler,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
        }
    }

    pub async fn start(&mut self, stop_send: Sender<bool>) {
        self.init_cache();

        let data_route = Arc::new(DataRoute::new(
            self.rocksdb_engine_handler.clone(),
            self.cluster_cache.clone(),
            self.engine_cache.clone(),
            self.mqtt_cache.clone(),
        ));

        self.start_call_thread();

        let openraft_node = create_raft_node(self.client_pool.clone(), data_route).await;

        let placement_center_storage = Arc::new(RaftMachineApply::new(openraft_node.clone()));

        self.start_heartbeat(placement_center_storage.clone(), stop_send.clone());

        self.start_raft_machine(openraft_node.clone());

        self.start_prometheus();

        self.start_grpc_server(placement_center_storage.clone());

        self.monitoring_leader_transition(openraft_node.clone(), placement_center_storage.clone());

        self.awaiting_stop(stop_send).await;
    }

    pub fn monitoring_leader_transition(
        &self,
        raft: Raft<TypeConfig>,
        raft_machine_apply: Arc<RaftMachineApply>,
    ) {
        info!("Initiate Monitoring of Raft Leader Transitions");
        monitoring_leader_transition(
            &raft,
            self.rocksdb_engine_handler.clone(),
            self.cluster_cache.clone(),
            self.mqtt_cache.clone(),
            self.engine_cache.clone(),
            self.client_pool.clone(),
            raft_machine_apply,
        );
    }

    // Start Grpc Server
    pub fn start_grpc_server(&self, raft_machine_apply: Arc<RaftMachineApply>) {
        let cluster_cache = self.cluster_cache.clone();
        let engine_cache = self.engine_cache.clone();
        let mqtt_cache = self.mqtt_cache.clone();
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let client_pool = self.client_pool.clone();
        let journal_call_manager = self.journal_call_manager.clone();
        let mqtt_call_manager = self.mqtt_call_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = start_grpc_server(
                raft_machine_apply,
                cluster_cache,
                engine_cache,
                mqtt_cache,
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

    pub fn start_heartbeat(
        &self,
        raft_machine_apply: Arc<RaftMachineApply>,
        stop_send: Sender<bool>,
    ) {
        // start cluster node heartbeate check
        let ctrl = ClusterController::new(
            self.cluster_cache.clone(),
            raft_machine_apply.clone(),
            stop_send.clone(),
            self.client_pool.clone(),
            self.journal_call_manager.clone(),
            self.mqtt_call_manager.clone(),
        );
        tokio::spawn(async move {
            ctrl.start_node_heartbeat_check().await;
        });

        // start mqtt connector scheduler thread
        let mqtt_cache = self.mqtt_cache.clone();
        let call_manager = self.mqtt_call_manager.clone();
        let client_pool = self.client_pool.clone();
        let cluster_cache = self.cluster_cache.clone();
        tokio::spawn(async move {
            start_connector_scheduler(
                &raft_machine_apply,
                &call_manager,
                &client_pool,
                &mqtt_cache,
                &cluster_cache,
                stop_send,
            )
            .await;
        });
    }

    // Start Raft Status Machine
    fn start_raft_machine(&self, openraft_node: Raft<TypeConfig>) {
        tokio::spawn(async move {
            start_openraft_node(openraft_node).await;
        });
    }

    fn start_prometheus(&self) {
        let conf = placement_center_conf();
        if conf.prometheus.enable {
            tokio::spawn(async move {
                register_prometheus_export(conf.prometheus.port).await;
            });
        }
    }

    fn start_call_thread(&self) {
        let client_pool = self.client_pool.clone();
        let journal_all_manager = self.journal_call_manager.clone();
        tokio::spawn(async move {
            journal_call_thread_manager(&journal_all_manager, &client_pool).await;
        });

        let mqtt_all_manager = self.mqtt_call_manager.clone();
        let client_pool = self.client_pool.clone();
        tokio::spawn(async move {
            mqtt_call_thread_manager(&mqtt_all_manager, &client_pool).await;
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

    pub fn init_cache(&self) {
        if let Err(e) = load_journal_cache(&self.engine_cache, &self.rocksdb_engine_handler) {
            panic!("Failed to load Journal Cache,{e}");
        }

        if let Err(e) = load_mqtt_cache(
            &self.mqtt_cache,
            &self.rocksdb_engine_handler,
            &self.cluster_cache,
        ) {
            panic!("Failed to load Mqtt Cache,{e}");
        }
    }
}
