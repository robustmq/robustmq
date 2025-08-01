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

use crate::grpc::start_grpc_server;
use common_base::{metrics::register_prometheus_export, runtime::create_runtime};
use common_config::{broker::broker_config, config::BrokerConfig};
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use journal_server::{
    core::cache::CacheManager as JournalCacheManager, segment::manager::SegmentFileManager,
    server::connection_manager::ConnectionManager as JournalConnectionManager, JournalServer,
    JournalServerParams,
};
use mqtt_broker::{
    bridge::manager::ConnectorManager,
    broker::{MqttBrokerServer, MqttBrokerServerParams},
    common::metrics_cache::MetricsCacheManager,
    handler::{cache::CacheManager as MqttCacheManager, heartbeat::check_placement_center_status},
    security::AuthDriver,
    server::common::connection_manager::ConnectionManager as MqttConnectionManager,
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
    storage::rocksdb::{column_family_list, RocksDBEngine},
    PlacementCenterServer, PlacementCenterServerParams,
};
use schema_register::schema::SchemaRegisterManager;
use std::{sync::Arc, thread::sleep, time::Duration};
use tokio::{runtime::Runtime, signal, sync::broadcast};
use tracing::{error, info};

pub mod common;
mod grpc;
mod metrics;

pub struct BrokerServer {
    main_runtime: Runtime,
    place_params: Option<PlacementCenterServerParams>,
    mqtt_params: Option<MqttBrokerServerParams>,
    journal_params: Option<JournalServerParams>,
    client_pool: Arc<ClientPool>,
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
        let main_runtime = create_runtime("init_runtime", config.runtime.runtime_worker_threads);
        let mut place_params = None;

        if config.is_start_meta() {
            main_runtime.block_on(async {
                place_params =
                    Some(BrokerServer::build_placement_center(client_pool.clone()).await);
            });
        }

        let mqtt_params = if config.is_start_broker() {
            Some(BrokerServer::build_mqtt_server(client_pool.clone()))
        } else {
            None
        };

        let journal_params = if config.is_start_journal() {
            Some(BrokerServer::build_journal_server(client_pool.clone()))
        } else {
            None
        };

        BrokerServer {
            main_runtime,
            place_params,
            journal_params,
            config: config.clone(),
            mqtt_params,
            client_pool,
        }
    }
    pub fn start(&self) {
        // start grpc server
        let place_params = self.place_params.clone();
        let mqtt_params = self.mqtt_params.clone();
        let journal_params = self.journal_params.clone();
        let server_runtime =
            create_runtime("server-runtime", self.config.runtime.runtime_worker_threads);
        let grpc_port = self.config.grpc_port;
        server_runtime.spawn(async move {
            if let Err(e) =
                start_grpc_server(place_params, mqtt_params, journal_params, grpc_port).await
            {
                panic!("{e}")
            }
        });

        // start prometheus
        let prometheus_port = self.config.prometheus.port;
        if self.config.prometheus.enable {
            server_runtime.spawn(async move {
                register_prometheus_export(prometheus_port).await;
            });
        }

        // check grpc service ready
        sleep(Duration::from_secs(1));

        let mut place_stop_send = None;
        let mut mqtt_stop_send = None;
        let mut journal_stop_send = None;

        // start placement center
        let (stop_send, _) = broadcast::channel(2);
        let place_runtime =
            create_runtime("place-runtime", self.config.runtime.runtime_worker_threads);
        if let Some(params) = self.place_params.clone() {
            place_stop_send = Some(stop_send.clone());
            place_runtime.spawn(async move {
                let mut pc = PlacementCenterServer::new(params, stop_send.clone());
                pc.start().await;
            });
        }

        // check placement ready
        self.main_runtime.block_on(async {
            check_placement_center_status(self.client_pool.clone()).await;
        });

        // // start journal server
        // let (stop_send, _) = broadcast::channel(2);
        // let journal_runtime = create_runtime(
        //     "journal-runtime",
        //     self.config.runtime.runtime_worker_threads,
        // );
        // let daemon_runtime = create_runtime(
        //     "journal-daemon-runtime",
        //     self.config.runtime.runtime_worker_threads,
        // );
        // if let Some(params) = self.journal_params.clone() {
        //     let server =
        //         JournalServer::new(params, journal_runtime, daemon_runtime, stop_send.clone());
        //     server.start();
        //     journal_stop_send = Some(stop_send.clone());
        // }

        // check journal ready
        //todo

        // start mqtt server
        // let daemon_runtime = create_runtime(
        //     "broker-daemon-runtime",
        //     self.config.runtime.runtime_worker_threads,
        // );
        // let admin_runtime = create_runtime(
        //     "broker-admin-runtime",
        //     self.config.runtime.runtime_worker_threads,
        // );
        // let connector_runtime = create_runtime(
        //     "broker-connector-runtime",
        //     self.config.runtime.runtime_worker_threads,
        // );
        // let server_runtime = create_runtime(
        //     "broker-server-runtime",
        //     self.config.runtime.runtime_worker_threads,
        // );
        // let subscribe_runtime = create_runtime(
        //     "broker-subscribe-runtime",
        //     self.config.runtime.runtime_worker_threads,
        // );

        // let (stop_send, _) = broadcast::channel(2);
        // if let Some(params) = self.mqtt_params.clone() {
        //     mqtt_stop_send = Some(stop_send.clone());
        //     let server = MqttBrokerServer::new(
        //         daemon_runtime,
        //         connector_runtime,
        //         server_runtime,
        //         subscribe_runtime,
        //         admin_runtime,
        //         params,
        //         stop_send,
        //     );
        //     server.start();
        // }

        // awaiting stop
        self.awaiting_stop(place_stop_send, mqtt_stop_send, journal_stop_send);
    }

    async fn build_placement_center(client_pool: Arc<ClientPool>) -> PlacementCenterServerParams {
        let config = broker_config();
        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(
            &placement_center::storage::rocksdb::storage_data_fold(&config.rocksdb.data_path),
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
        let connection_manager = Arc::new(MqttConnectionManager::new(cache_manager.clone()));
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

    fn build_journal_server(client_pool: Arc<ClientPool>) -> JournalServerParams {
        let config = broker_config();
        let connection_manager = Arc::new(JournalConnectionManager::new());
        let cache_manager = Arc::new(JournalCacheManager::new());
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &journal_server::index::engine::storage_data_fold(&config.journal_storage.data_path),
            10000,
            column_family_list(),
        ));

        let segment_file_manager =
            Arc::new(SegmentFileManager::new(rocksdb_engine_handler.clone()));

        JournalServerParams {
            cache_manager,
            client_pool,
            connection_manager,
            segment_file_manager,
            rocksdb_engine_handler,
        }
    }

    pub fn awaiting_stop(
        &self,
        place_stop: Option<broadcast::Sender<bool>>,
        mqtt_stop: Option<broadcast::Sender<bool>>,
        journal_stop: Option<broadcast::Sender<bool>>,
    ) {
        self.main_runtime.block_on(async {
            // Wait for all the request packets in the TCP Channel to be processed completely before starting to stop other processing threads.
            signal::ctrl_c().await.expect("failed to listen for event");
            info!(
                "{}",
                "When ctrl + c is received, the service starts to stop"
            );

            if let Some(sx) = mqtt_stop {
                if let Err(e) = sx.send(true) {
                    error!("mqtt stop signal, error message:{}", e);
                }
                sleep(Duration::from_secs(3));
            }

            if let Some(sx) = journal_stop {
                if let Err(e) = sx.send(true) {
                    error!("journal stop signal, error message{}", e);
                }
                sleep(Duration::from_secs(3));
            }

            if let Some(sx) = place_stop {
                if let Err(e) = sx.send(true) {
                    error!("place stop signal, error message{}", e);
                }
            }
            sleep(Duration::from_secs(10));
        });
    }
}
