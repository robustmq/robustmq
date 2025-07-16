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

use std::{sync::Arc, thread::sleep, time::Duration};

use crate::grpc::start_grpc_server;
use common_base::runtime::create_runtime;
use common_config::broker::{broker_config, config::BrokerConfig};
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use mqtt_broker::{
    bridge::manager::ConnectorManager,
    broker::{MqttBrokerServer, MqttBrokerServerParams},
    common::metrics_cache::MetricsCacheManager,
    handler::{cache::CacheManager as MqttCacheManager, heartbeat::check_placement_center_status},
    security::AuthDriver,
    server::common::connection_manager::ConnectionManager,
    storage::message::build_message_storage_driver,
    subscribe::manager::SubscribeManager,
};
use openraft::Raft;
use placement_center::{
    controller::{
        journal::call_node::JournalInnerCallManager, mqtt::call_broker::MQTTInnerCallManager,
    },
    core::cache::CacheManager as PlacementCacheManager,
    raft::{
        raft_node::create_raft_node,
        route::{apply::StorageDriver, DataRoute},
        type_config::TypeConfig,
    },
    storage::rocksdb::{column_family_list, storage_data_fold, RocksDBEngine},
    PlacementCenterServer, PlacementCenterServerParams,
};
use schema_register::schema::SchemaRegisterManager;
use tokio::{runtime::Runtime, signal, sync::broadcast};
use tracing::{error, info};

pub mod common;
mod grpc;
mod metrics;

pub struct BrokerServer {
    place_params: PlacementCenterServerParams,
    mqtt_params: MqttBrokerServerParams,
    runtime: Runtime,
    config: BrokerConfig,
}

impl Default for BrokerServer {
    fn default() -> Self {
        Self::new()
    }
}

impl BrokerServer {
    pub fn new() -> Self {
        let config = broker_config();
        let client_pool = Arc::new(ClientPool::new(100));
        let runtime = create_runtime("daemon-runtime", config.runtime.runtime_worker_threads);
        let mut place_params = None;
        let raw_client_pool = client_pool.clone();
        runtime.block_on(async {
            place_params = Some(BrokerServer::build_placement_center(raw_client_pool).await);
        });
        let mqtt_params = BrokerServer::build_mqtt_server(client_pool);

        BrokerServer {
            place_params: place_params.unwrap(),
            runtime,
            config: config.clone(),
            mqtt_params,
        }
    }
    pub fn start(&self) {
        self.start_grpc_server();
        sleep(Duration::from_secs(3));

        let place_stop = self.start_placement_center();

        // check placement ready
        self.runtime.block_on(async move {
            check_placement_center_status(self.place_params.client_pool.clone()).await
        });

        // start journal server

        // start mqtt server
        let mqtt_stop = self.start_mqtt_server();

        self.awaiting_stop(place_stop, mqtt_stop);
    }

    fn start_placement_center(&self) -> broadcast::Sender<bool> {
        let (place_stop_send, _) = broadcast::channel(2);
        let runtime = create_runtime(
            "placement-runtime",
            self.config.runtime.runtime_worker_threads,
        );
        let place_params = self.place_params.clone();
        let raw_place_stop_send = place_stop_send.clone();
        runtime.spawn(async move {
            let mut pc = PlacementCenterServer::new(place_params.clone());
            pc.start(raw_place_stop_send).await;
        });
        place_stop_send
    }

    fn start_mqtt_server(&self) -> broadcast::Sender<bool> {
        let (mqtt_stop_send, _) = broadcast::channel(2);
        let server = MqttBrokerServer::new(self.mqtt_params.clone(), mqtt_stop_send.clone());
        server.start(mqtt_stop_send.clone());
        mqtt_stop_send
    }

    fn start_grpc_server(&self) {
        let place_params = self.place_params.clone();
        let mqtt_params = self.mqtt_params.clone();
        let config = self.config.clone();
        self.runtime.spawn(async move {
            if let Err(e) = start_grpc_server(&place_params, &mqtt_params, config.grpc_port).await {
                panic!("{e}")
            }
        });
    }

    async fn build_placement_center(client_pool: Arc<ClientPool>) -> PlacementCenterServerParams {
        let config = broker_config();
        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));

        let cache_manager = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));

        let journal_call_manager = Arc::new(JournalInnerCallManager::new(cache_manager.clone()));
        let mqtt_call_manager = Arc::new(MQTTInnerCallManager::new(cache_manager.clone()));

        let data_route = Arc::new(DataRoute::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
        ));
        let raf_node: Raft<TypeConfig> = create_raft_node(client_pool.clone(), data_route).await;
        let storage_driver: Arc<StorageDriver> = Arc::new(StorageDriver::new(raf_node.clone()));
        PlacementCenterServerParams {
            cache_manager,
            rocksdb_engine_handler,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
            raf_node,
            storage_driver,
        }
    }

    fn build_mqtt_server(client_pool: Arc<ClientPool>) -> MqttBrokerServerParams {
        let config = broker_config();
        let cache_manager = Arc::new(MqttCacheManager::new(
            client_pool.clone(),
            config.cluster_name.clone(),
        ));

        let storage_driver = match build_message_storage_driver() {
            Ok(storage) => storage,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let arc_storage_driver = Arc::new(storage_driver);
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let connector_manager = Arc::new(ConnectorManager::new());
        let connection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));
        let auth_driver = Arc::new(AuthDriver::new(cache_manager.clone(), client_pool.clone()));
        let delay_message_manager = Arc::new(DelayMessageManager::new(
            config.cluster_name.clone(),
            1,
            arc_storage_driver.clone(),
        ));
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let schema_manager = Arc::new(SchemaRegisterManager::new());

        MqttBrokerServerParams {
            cache_manager,
            client_pool,
            message_storage_adapter: arc_storage_driver,
            subscribe_manager,
            connection_manager,
            connector_manager,
            auth_driver,
            delay_message_manager,
            schema_manager,
            metrics_cache_manager,
        }
    }

    pub fn awaiting_stop(
        &self,
        place_stop: broadcast::Sender<bool>,
        mqtt_stop: broadcast::Sender<bool>,
    ) {
        self.runtime.block_on(async move {
            // Wait for all the request packets in the TCP Channel to be processed completely before starting to stop other processing threads.
            signal::ctrl_c().await.expect("failed to listen for event");
            info!(
                "{}",
                "When ctrl + c is received, the service starts to stop"
            );
            if let Err(e) = mqtt_stop.send(true) {
                error!("{}", e);
            }
            // todo

            if let Err(e) = place_stop.send(true) {
                error!("{}", e);
            }
        });
    }
}
