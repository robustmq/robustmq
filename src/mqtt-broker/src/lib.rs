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

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::runtime::create_runtime;
use common_base::tools::now_second;
use grpc_clients::poll::ClientPool;
use handler::cache::CacheManager;
use handler::heartbreat::report_heartbeat;
use handler::keep_alive::ClientKeepAlive;
use lazy_static::lazy_static;
use log::{error, info};
use observability::start_opservability;
use security::AuthDriver;
use server::connection_manager::ConnectionManager;
use server::grpc::server::GrpcServer;
use server::http::server::{start_http_server, HttpServerState};
use server::tcp::server::start_tcp_server;
use server::websocket::server::{websocket_server, websockets_server, WebSocketServerState};
use storage::cluster::ClusterStorage;
use storage_adapter::local_rocksdb::RocksDBStorageAdapter;
use storage_adapter::memory::MemoryStorageAdapter;
use storage_adapter::mysql::MySQLStorageAdapter;
use storage_adapter::storage::StorageAdapter;
use storage_adapter::StorageType;
use subscribe::sub_exclusive::SubscribeExclusive;
use subscribe::sub_share_follower::SubscribeShareFollower;
use subscribe::sub_share_leader::SubscribeShareLeader;
use subscribe::subscribe_manager::SubscribeManager;
use third_driver::mysql::build_mysql_conn_pool;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::broadcast::{self};
use tokio::time::sleep;

lazy_static! {
    pub static ref BROKER_START_TIME: u64 = now_second();
}

pub mod handler;
mod observability;
mod security;
mod server;
pub mod storage;
mod subscribe;

pub fn start_mqtt_broker_server(stop_send: broadcast::Sender<bool>) {
    let conf = broker_mqtt_conf();
    let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(5));
    let metadata_cache = Arc::new(CacheManager::new(
        client_poll.clone(),
        conf.cluster_name.clone(),
    ));
    // let storage_type = conf.storage.storage_type.clone();
    let storage_type = StorageType::from_str(conf.storage.storage_type.as_str())
        .expect("Storage type not supported");
    match storage_type {
        StorageType::Memory => {
            let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());
            let server = MqttBroker::new(client_poll, message_storage_adapter, metadata_cache);
            server.start(stop_send);
        }
        StorageType::Mysql => {
            if conf.storage.mysql_addr.is_empty() {
                panic!("storaget type is [mysql],[storage.mysql_addr] cannot be empty");
            }
            let pool = build_mysql_conn_pool(&conf.storage.mysql_addr).unwrap();
            let message_storage_adapter = Arc::new(MySQLStorageAdapter::new(pool.clone()));
            let server: MqttBroker<MySQLStorageAdapter> =
                MqttBroker::new(client_poll, message_storage_adapter, metadata_cache);
            server.start(stop_send);
        }
        StorageType::RocksDB => {
            if conf.storage.rocksdb_data_path.is_empty() {
                panic!("storaget type is [rocksdb],[storage.rocksdb_path] cannot be empty");
            }
            let message_storage_adapter = Arc::new(RocksDBStorageAdapter::new(
                conf.storage.rocksdb_data_path.as_str(),
                conf.storage.rocksdb_max_open_files.unwrap_or(10000),
            ));
            let server = MqttBroker::new(client_poll, message_storage_adapter, metadata_cache);
            server.start(stop_send);
        }
        _ => {
            panic!("Message data storage type configuration error, optional :mysql, memory");
        }
    }
}

pub struct MqttBroker<S> {
    cache_manager: Arc<CacheManager>,
    runtime: Runtime,
    client_poll: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    auth_driver: Arc<AuthDriver>,
}

impl<S> MqttBroker<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        client_poll: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        let conf = broker_mqtt_conf();
        let runtime = create_runtime(
            "storage-engine-server-runtime",
            conf.system.runtime_worker_threads,
        );

        let subscribe_manager = Arc::new(SubscribeManager::new(
            cache_manager.clone(),
            client_poll.clone(),
        ));

        let connection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));

        let auth_driver = Arc::new(AuthDriver::new(cache_manager.clone(), client_poll.clone()));
        MqttBroker {
            runtime,
            cache_manager,
            client_poll,
            message_storage_adapter,
            subscribe_manager,
            connection_manager,
            auth_driver,
        }
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        self.register_node();
        self.start_grpc_server();
        self.start_mqtt_server(stop_send.clone());
        self.start_http_server();
        self.start_websocket_server(stop_send.clone());
        self.start_keep_alive_thread(stop_send.clone());
        self.start_cluster_heartbeat_report(stop_send.clone());
        self.start_push_server();
        self.start_system_topic_thread(stop_send.clone());
        self.awaiting_stop(stop_send);
    }

    fn start_mqtt_server(&self, stop_send: broadcast::Sender<bool>) {
        let cache = self.cache_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let client_poll = self.client_poll.clone();
        let connection_manager = self.connection_manager.clone();
        let auth_driver = self.auth_driver.clone();

        self.runtime.spawn(async move {
            start_tcp_server(
                subscribe_manager,
                cache,
                connection_manager,
                message_storage_adapter,
                client_poll,
                stop_send,
                auth_driver,
            )
            .await
        });
    }

    fn start_grpc_server(&self) {
        let conf = broker_mqtt_conf();
        let server = GrpcServer::new(
            conf.grpc_port,
            self.cache_manager.clone(),
            self.subscribe_manager.clone(),
            self.client_poll.clone(),
            self.message_storage_adapter.clone(),
        );
        self.runtime.spawn(async move {
            match server.start().await {
                Ok(()) => {}
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            }
        });
    }

    fn start_http_server(&self) {
        let http_state = HttpServerState::new();
        self.runtime.spawn(async move {
            match start_http_server(http_state).await {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            }
        });
    }

    fn start_websocket_server(&self, stop_send: broadcast::Sender<bool>) {
        let ws_state = WebSocketServerState::new(
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            self.connection_manager.clone(),
            self.message_storage_adapter.clone(),
            self.client_poll.clone(),
            self.auth_driver.clone(),
            stop_send.clone(),
        );
        self.runtime
            .spawn(async move { websocket_server(ws_state).await });

        let ws_state = WebSocketServerState::new(
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            self.connection_manager.clone(),
            self.message_storage_adapter.clone(),
            self.client_poll.clone(),
            self.auth_driver.clone(),
            stop_send.clone(),
        );

        self.runtime
            .spawn(async move { websockets_server(ws_state).await });
    }

    fn start_cluster_heartbeat_report(&self, stop_send: broadcast::Sender<bool>) {
        let client_poll = self.client_poll.clone();
        self.runtime.spawn(async move {
            report_heartbeat(client_poll, stop_send).await;
        });
    }

    fn start_push_server(&self) {
        let subscribe_manager = self.subscribe_manager.clone();
        self.runtime.spawn(async move {
            subscribe_manager.start().await;
        });

        let exclusive_sub = SubscribeExclusive::new(
            self.message_storage_adapter.clone(),
            self.cache_manager.clone(),
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.client_poll.clone(),
        );

        self.runtime.spawn(async move {
            exclusive_sub.start().await;
        });

        let leader_sub = SubscribeShareLeader::new(
            self.subscribe_manager.clone(),
            self.message_storage_adapter.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            self.client_poll.clone(),
        );

        self.runtime.spawn(async move {
            leader_sub.start().await;
        });

        let follower_sub = SubscribeShareFollower::new(
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            self.client_poll.clone(),
        );

        self.runtime.spawn(async move {
            follower_sub.start().await;
        });
    }

    fn start_keep_alive_thread(&self, stop_send: broadcast::Sender<bool>) {
        let mut keep_alive = ClientKeepAlive::new(
            self.client_poll.clone(),
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            stop_send,
        );
        self.runtime.spawn(async move {
            keep_alive.start_heartbeat_check().await;
        });
    }

    fn start_system_topic_thread(&self, stop_send: broadcast::Sender<bool>) {
        let cache_manager = self.cache_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let client_poll = self.client_poll.clone();
        self.runtime.spawn(async move {
            start_opservability(
                cache_manager,
                message_storage_adapter,
                client_poll,
                stop_send,
            )
            .await;
        });
    }

    pub fn awaiting_stop(&self, stop_send: broadcast::Sender<bool>) {
        self.runtime.spawn(async move {
            sleep(Duration::from_millis(5)).await;
            info!("MQTT Broker service started successfully...");
        });

        // Wait for the stop signal
        self.runtime.block_on(async move {
            signal::ctrl_c().await.expect("failed to listen for event");
            match stop_send.send(true) {
                Ok(_) => {
                    info!(
                        "{}",
                        "When ctrl + c is received, the service starts to stop"
                    );
                    self.stop_server().await;
                }
                Err(_) => {
                    error!("Failed to send stop signal");
                }
            }
        });

        // todo tokio runtime shutdown
    }

    fn register_node(&self) {
        let metadata_cache = self.cache_manager.clone();
        let client_poll = self.client_poll.clone();
        let auth_driver = self.auth_driver.clone();
        self.runtime.block_on(async move {
            metadata_cache.init_system_user().await;
            metadata_cache.load_metadata_cache(auth_driver).await;

            let cluster_storage = ClusterStorage::new(client_poll.clone());
            let config = broker_mqtt_conf();
            match cluster_storage.register_node(config).await {
                Ok(_) => {
                    info!("Node {} has been successfully registered", config.broker_id);
                }
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            }
        });
    }

    async fn stop_server(&self) {
        let cluster_storage = ClusterStorage::new(self.client_poll.clone());
        let config = broker_mqtt_conf();
        match cluster_storage.unregister_node(config).await {
            Ok(()) => {
                info!("Node {} exits successfully", config.broker_id);
            }
            Err(e) => {
                error!("{}", e.to_string());
            }
        }
        self.connection_manager.close_all_connect().await;
    }
}
