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

use bridge::core::start_connector_thread;
use bridge::manager::ConnectorManager;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::metrics::register_prometheus_export;
use common_base::runtime::create_runtime;
use common_base::tools::now_second;
use delay_message::{start_delay_message_manager, DelayMessageManager};
use grpc_clients::pool::ClientPool;
use handler::acl::UpdateAclCache;
use handler::cache::CacheManager;
use handler::cache_update::load_metadata_cache;
use handler::heartbreat::{register_node, report_heartbeat};
use handler::keep_alive::ClientKeepAlive;
use handler::sub_parse_topic::start_parse_subscribe_by_new_topic_thread;
use handler::user::{init_system_user, UpdateUserCache};
use lazy_static::lazy_static;
use observability::start_opservability;
use pprof_monitor::pprof_monitor::start_pprof_monitor;
use schema_register::schema::SchemaRegisterManager;
use security::AuthDriver;
use server::connection_manager::ConnectionManager;
use server::grpc::server::GrpcServer;
use server::tcp::server::start_tcp_server;
use server::websocket::server::{websocket_server, websockets_server, WebSocketServerState};
use storage::cluster::ClusterStorage;
use storage_adapter::memory::MemoryStorageAdapter;
use tracing::{error, info};
// use storage_adapter::mysql::MySQLStorageAdapter;
// use storage_adapter::rocksdb::RocksDBStorageAdapter;
use crate::handler::flapping_detect::UpdateFlappingDetectCache;
use crate::server::quic::server::start_quic_server;
use storage_adapter::storage::StorageAdapter;
use storage_adapter::StorageType;
use subscribe::exclusive_push::ExclusivePush;
use subscribe::share_follower_resub::ShareFollowerResub;
use subscribe::share_leader_push::ShareLeaderPush;
use subscribe::subscribe_manager::SubscribeManager;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::broadcast::{self};
use tokio::time::sleep;

lazy_static! {
    pub static ref BROKER_START_TIME: u64 = now_second();
}

pub mod admin;
pub mod bridge;
pub mod handler;
pub mod observability;
pub mod security;
pub mod server;
pub mod storage;
mod subscribe;

pub fn start_mqtt_broker_server(stop_send: broadcast::Sender<bool>) {
    let conf = broker_mqtt_conf();
    let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(100));
    let metadata_cache = Arc::new(CacheManager::new(
        client_pool.clone(),
        conf.cluster_name.clone(),
    ));
    // let storage_type = conf.storage.storage_type.clone();
    let storage_type = StorageType::from_str(conf.storage.storage_type.as_str())
        .expect("Storage type not supported");
    match storage_type {
        StorageType::Memory => {
            let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());
            let server = MqttBroker::new(client_pool, message_storage_adapter, metadata_cache);
            server.start(stop_send);
        }
        // StorageType::Mysql => {
        //     if conf.storage.mysql_addr.is_empty() {
        //         panic!("storaget type is [mysql],[storage.mysql_addr] cannot be empty");
        //     }
        //     let pool = build_mysql_conn_pool(&conf.storage.mysql_addr).unwrap();
        //     let message_storage_adapter = Arc::new(MySQLStorageAdapter::new(pool.clone()));
        //     let server: MqttBroker<MySQLStorageAdapter> =
        //         MqttBroker::new(client_pool, message_storage_adapter, metadata_cache);
        //     server.start(stop_send);
        // }
        // StorageType::RocksDB => {
        //     if conf.storage.rocksdb_data_path.is_empty() {
        //         panic!("storaget type is [rocksdb],[storage.rocksdb_path] cannot be empty");
        //     }
        //     let message_storage_adapter = Arc::new(RocksDBStorageAdapter::new(
        //         conf.storage.rocksdb_data_path.as_str(),
        //         conf.storage.rocksdb_max_open_files.unwrap_or(10000),
        //     ));
        //     let server = MqttBroker::new(client_pool, message_storage_adapter, metadata_cache);
        //     server.start(stop_send);
        // }
        _ => {
            panic!("Message data storage type configuration error, optional :mysql, memory");
        }
    }
}

pub struct MqttBroker<S> {
    cache_manager: Arc<CacheManager>,
    daemon_runtime: Runtime,
    connector_runtime: Runtime,
    publish_runtime: Runtime,
    subscribe_runtime: Runtime,
    grpc_runtime: Runtime,
    client_pool: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    connector_manager: Arc<ConnectorManager>,
    auth_driver: Arc<AuthDriver>,
    delay_message_manager: Arc<DelayMessageManager<S>>,
    schema_manager: Arc<SchemaRegisterManager>,
}

impl<S> MqttBroker<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        client_pool: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        let conf = broker_mqtt_conf();
        let daemon_runtime = create_runtime("daemon-runtime", conf.system.runtime_worker_threads);

        let connector_runtime =
            create_runtime("connector-runtime", conf.system.runtime_worker_threads);
        let publish_runtime = create_runtime("publish-runtime", conf.system.runtime_worker_threads);
        let subscribe_runtime =
            create_runtime("subscribe-runtime", conf.system.runtime_worker_threads);
        let grpc_runtime = create_runtime("grpc-runtime", conf.system.runtime_worker_threads);

        let subscribe_manager = Arc::new(SubscribeManager::new());
        let connector_manager = Arc::new(ConnectorManager::new());
        let connection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));
        let auth_driver = Arc::new(AuthDriver::new(cache_manager.clone(), client_pool.clone()));
        let delay_message_manager = Arc::new(DelayMessageManager::new(
            conf.cluster_name.clone(),
            1,
            message_storage_adapter.clone(),
        ));
        let schema_manager = Arc::new(SchemaRegisterManager::new());
        MqttBroker {
            daemon_runtime,
            connector_runtime,
            publish_runtime,
            subscribe_runtime,
            grpc_runtime,
            cache_manager,
            client_pool,
            message_storage_adapter,
            subscribe_manager,
            connector_manager,
            connection_manager,
            auth_driver,
            delay_message_manager,
            schema_manager,
        }
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        // daemon runtime
        self.start_tracer_provider();
        self.register_node();
        self.start_cluster_heartbeat_report(stop_send.clone());
        self.start_keep_alive_thread(stop_send.clone());
        self.start_delay_message_thread();
        self.start_update_cache_thread(stop_send.clone());
        self.start_system_topic_thread(stop_send.clone());
        self.start_prometheus();
        self.start_pprof_monitor();

        // grpc runtime
        self.start_grpc_server();

        // connector runtime
        self.start_connector_thread(stop_send.clone());

        // publish runtime
        self.start_mqtt_server(stop_send.clone());
        self.start_quic_server(stop_send.clone());
        self.start_websocket_server(stop_send.clone());

        // subscribe runtime
        self.start_subscribe_push(stop_send.clone());

        self.awaiting_stop(stop_send);
    }

    fn start_tracer_provider(&self) {
        self.daemon_runtime.spawn(async move {
            common_base::telemetry::trace::init_tracer_provider(broker_mqtt_conf()).await;
        });
    }
    fn start_mqtt_server(&self, stop_send: broadcast::Sender<bool>) {
        let cache = self.cache_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let client_pool = self.client_pool.clone();
        let connection_manager = self.connection_manager.clone();
        let auth_driver = self.auth_driver.clone();
        let delay_message_manager = self.delay_message_manager.clone();
        let schema_manager = self.schema_manager.clone();

        self.publish_runtime.spawn(async move {
            start_tcp_server(
                subscribe_manager,
                cache,
                connection_manager,
                message_storage_adapter,
                delay_message_manager,
                schema_manager,
                client_pool,
                stop_send,
                auth_driver,
            )
            .await
        });
    }

    fn start_prometheus(&self) {
        let conf = broker_mqtt_conf();
        if conf.prometheus.enable {
            self.daemon_runtime.spawn(async move {
                register_prometheus_export(conf.prometheus.port).await;
            });
        }
    }

    fn start_pprof_monitor(&self) {
        let conf = broker_mqtt_conf();
        if conf.pprof.enable {
            self.daemon_runtime.spawn(async move {
                start_pprof_monitor(conf.pprof.port, conf.pprof.frequency).await;
            });
        }
    }

    fn start_quic_server(&self, stop_send: broadcast::Sender<bool>) {
        let cache = self.cache_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let client_pool = self.client_pool.clone();
        let connection_manager = self.connection_manager.clone();
        let auth_driver = self.auth_driver.clone();
        let delay_message_manager = self.delay_message_manager.clone();
        let schema_manager = self.schema_manager.clone();
        self.publish_runtime.spawn(async move {
            start_quic_server(
                subscribe_manager,
                cache,
                connection_manager,
                message_storage_adapter,
                delay_message_manager,
                client_pool,
                stop_send,
                auth_driver,
                schema_manager,
            )
            .await
        });
    }

    fn start_grpc_server(&self) {
        let conf = broker_mqtt_conf();
        let server = GrpcServer::new(
            conf.grpc_port,
            self.cache_manager.clone(),
            self.connector_manager.clone(),
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.schema_manager.clone(),
            self.client_pool.clone(),
            self.message_storage_adapter.clone(),
        );
        self.grpc_runtime.spawn(async move {
            if let Err(e) = server.start().await {
                panic!("{}", e.to_string());
            }
        });
    }

    fn start_websocket_server(&self, stop_send: broadcast::Sender<bool>) {
        let ws_state = WebSocketServerState::new(
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            self.connection_manager.clone(),
            self.message_storage_adapter.clone(),
            self.delay_message_manager.clone(),
            self.schema_manager.clone(),
            self.client_pool.clone(),
            self.auth_driver.clone(),
            stop_send.clone(),
        );
        self.daemon_runtime
            .spawn(async move { websocket_server(ws_state).await });

        let ws_state = WebSocketServerState::new(
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            self.connection_manager.clone(),
            self.message_storage_adapter.clone(),
            self.delay_message_manager.clone(),
            self.schema_manager.clone(),
            self.client_pool.clone(),
            self.auth_driver.clone(),
            stop_send.clone(),
        );

        self.publish_runtime
            .spawn(async move { websockets_server(ws_state).await });
    }

    fn start_cluster_heartbeat_report(&self, stop_send: broadcast::Sender<bool>) {
        let client_pool = self.client_pool.clone();
        self.daemon_runtime.spawn(async move {
            let conf = broker_mqtt_conf();
            report_heartbeat(&client_pool, &conf.heartbeat_timeout, stop_send).await;
        });
    }

    fn start_connector_thread(&self, stop_send: broadcast::Sender<bool>) {
        let message_storage = self.message_storage_adapter.clone();
        let connector_manager = self.connector_manager.clone();
        self.connector_runtime.spawn(async move {
            start_connector_thread(message_storage, connector_manager, stop_send).await;
        });
    }

    fn start_subscribe_push(&self, stop_send: broadcast::Sender<bool>) {
        let subscribe_manager = self.subscribe_manager.clone();
        let client_pool = self.client_pool.clone();
        let metadata_cache = self.cache_manager.clone();

        self.subscribe_runtime.spawn(async move {
            start_parse_subscribe_by_new_topic_thread(
                &client_pool,
                &metadata_cache,
                &subscribe_manager,
                stop_send,
            )
            .await;
        });

        let exclusive_sub = ExclusivePush::new(
            self.message_storage_adapter.clone(),
            self.cache_manager.clone(),
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
        );

        self.subscribe_runtime.spawn(async move {
            exclusive_sub.start().await;
        });

        let leader_sub = ShareLeaderPush::new(
            self.subscribe_manager.clone(),
            self.message_storage_adapter.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
        );

        self.subscribe_runtime.spawn(async move {
            leader_sub.start().await;
        });

        let follower_sub = ShareFollowerResub::new(
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            self.client_pool.clone(),
        );

        self.subscribe_runtime.spawn(async move {
            follower_sub.start().await;
        });
    }

    fn start_keep_alive_thread(&self, stop_send: broadcast::Sender<bool>) {
        let mut keep_alive = ClientKeepAlive::new(
            self.client_pool.clone(),
            self.connection_manager.clone(),
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            stop_send,
        );
        self.daemon_runtime.spawn(async move {
            keep_alive.start_heartbeat_check().await;
        });
    }

    fn start_delay_message_thread(&self) {
        let delay_message_manager = self.delay_message_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        self.daemon_runtime.spawn(async move {
            let conf = broker_mqtt_conf();
            if let Err(e) = start_delay_message_manager(
                &delay_message_manager,
                &message_storage_adapter,
                &conf.cluster_name,
                delay_message_manager.get_shard_num(),
            )
            .await
            {
                panic!("{}", e.to_string());
            }
        });
    }

    fn start_update_cache_thread(&self, stop_send: broadcast::Sender<bool>) {
        let update_user_cache = UpdateUserCache::new(stop_send.clone(), self.auth_driver.clone());

        self.daemon_runtime.spawn(async move {
            update_user_cache.start_update().await;
        });

        let update_acl_cache = UpdateAclCache::new(stop_send.clone(), self.auth_driver.clone());

        self.daemon_runtime.spawn(async move {
            update_acl_cache.start_update().await;
        });

        let update_flapping_detect_cache =
            UpdateFlappingDetectCache::new(stop_send.clone(), self.cache_manager.clone());
        self.daemon_runtime.spawn(async move {
            update_flapping_detect_cache.start_update().await;
        });
    }

    fn start_system_topic_thread(&self, stop_send: broadcast::Sender<bool>) {
        let cache_manager = self.cache_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let client_pool = self.client_pool.clone();
        self.daemon_runtime.spawn(async move {
            start_opservability(
                cache_manager,
                message_storage_adapter,
                client_pool,
                stop_send,
            )
            .await;
        });
    }

    pub fn awaiting_stop(&self, stop_send: broadcast::Sender<bool>) {
        self.daemon_runtime.spawn(async move {
            sleep(Duration::from_millis(5)).await;
            info!("MQTT Broker service started successfully...");
        });

        // Wait for the stop signal
        self.daemon_runtime.block_on(async move {
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
        self.daemon_runtime.block_on(async move {
            init_system_user(&self.cache_manager, &self.client_pool).await;
            load_metadata_cache(
                &self.cache_manager,
                &self.client_pool,
                &self.auth_driver,
                &self.connector_manager,
                &self.schema_manager,
            )
            .await;

            let config = broker_mqtt_conf();
            match register_node(&self.client_pool).await {
                Ok(()) => {
                    info!("Node {} has been successfully registered", config.broker_id);
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
        });
    }

    async fn stop_server(&self) {
        let cluster_storage = ClusterStorage::new(self.client_pool.clone());
        let config = broker_mqtt_conf();
        common_base::telemetry::trace::stop_tracer_provider().await;
        let _ = self.delay_message_manager.stop().await;
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
