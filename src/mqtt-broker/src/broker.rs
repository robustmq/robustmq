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
use crate::bridge::core::start_connector_thread;
use crate::bridge::manager::ConnectorManager;
use crate::common::metrics_cache::{metrics_gc_thread, metrics_record_thread, MetricsCacheManager};
use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::CacheManager;
use crate::handler::dynamic_cache::load_metadata_cache;
use crate::handler::flapping_detect::UpdateFlappingDetectCache;
use crate::handler::heartbeat::{check_placement_center_status, register_node, report_heartbeat};
use crate::handler::keep_alive::ClientKeepAlive;
use crate::handler::sub_parse_topic::start_parse_subscribe_by_new_topic_thread;
use crate::observability::start_observability;
use crate::security::auth::super_user::init_system_user;
use crate::security::storage::sync::sync_auth_storage_info;
use crate::security::AuthDriver;
use crate::server::common::connection_manager::ConnectionManager;
use crate::server::grpc::server::GrpcServer;
use crate::server::quic::server::start_quic_server;
use crate::server::server::Server;
use crate::server::websocket::server::{websocket_server, websockets_server, WebSocketServerState};
use crate::storage::cluster::ClusterStorage;
use crate::subscribe::exclusive::ExclusivePush;
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::share::follower::ShareFollowerResub;
use crate::subscribe::share::leader::ShareLeaderPush;
use common_base::metrics::register_prometheus_export;
use common_base::runtime::create_runtime;
use common_config::mqtt::broker_mqtt_conf;
use delay_message::{start_delay_message_manager, DelayMessageManager};
use grpc_clients::pool::ClientPool;
use pprof_monitor::pprof_monitor::start_pprof_monitor;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::broadcast::{self};
use tokio::time::sleep;
use tracing::{error, info};

pub struct MqttBroker {
    cache_manager: Arc<CacheManager>,
    daemon_runtime: Runtime,
    connector_runtime: Runtime,
    server_runtime: Runtime,
    subscribe_runtime: Runtime,
    admin_runtime: Runtime,
    client_pool: Arc<ClientPool>,
    message_storage_adapter: ArcStorageAdapter,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    connector_manager: Arc<ConnectorManager>,
    auth_driver: Arc<AuthDriver>,
    delay_message_manager: Arc<DelayMessageManager>,
    schema_manager: Arc<SchemaRegisterManager>,
    metrics_cache_manager: Arc<MetricsCacheManager>,
    server: Arc<Server>,
}

impl MqttBroker {
    pub fn new(
        client_pool: Arc<ClientPool>,
        message_storage_adapter: ArcStorageAdapter,
        cache_manager: Arc<CacheManager>,
        stop_sx: broadcast::Sender<bool>,
    ) -> Self {
        let conf = broker_mqtt_conf();
        let daemon_runtime = create_runtime("daemon-runtime", conf.system.runtime_worker_threads);
        let admin_runtime = create_runtime("admin-runtime", conf.system.runtime_worker_threads);

        let connector_runtime =
            create_runtime("connector-runtime", conf.system.runtime_worker_threads);

        let server_runtime = create_runtime("server-runtime", conf.system.runtime_worker_threads);
        let subscribe_runtime =
            create_runtime("subscribe-runtime", conf.system.runtime_worker_threads);

        let subscribe_manager = Arc::new(SubscribeManager::new());
        let connector_manager = Arc::new(ConnectorManager::new());
        let connection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));
        let auth_driver = Arc::new(AuthDriver::new(cache_manager.clone(), client_pool.clone()));
        let delay_message_manager = Arc::new(DelayMessageManager::new(
            conf.cluster_name.clone(),
            1,
            message_storage_adapter.clone(),
        ));
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let schema_manager = Arc::new(SchemaRegisterManager::new());
        let server = Arc::new(Server::new(
            subscribe_manager.clone(),
            cache_manager.clone(),
            connection_manager.clone(),
            message_storage_adapter.clone(),
            delay_message_manager.clone(),
            schema_manager.clone(),
            client_pool.clone(),
            stop_sx,
            auth_driver.clone(),
        ));
        MqttBroker {
            daemon_runtime,
            connector_runtime,
            server_runtime,
            subscribe_runtime,
            admin_runtime,
            cache_manager,
            client_pool,
            message_storage_adapter,
            subscribe_manager,
            connector_manager,
            connection_manager,
            auth_driver,
            delay_message_manager,
            schema_manager,
            server,
            metrics_cache_manager,
        }
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        self.start_init();

        self.start_daemon_thread(stop_send.clone());

        self.start_delay_message_thread();

        self.start_connector_thread(stop_send.clone());

        self.start_subscribe_push(stop_send.clone());

        self.start_server(stop_send.clone());

        self.awaiting_stop(stop_send);
    }

    fn start_daemon_thread(&self, stop_send: broadcast::Sender<bool>) {
        // client keep alive
        let raw_stop_send = stop_send.clone();
        let keep_alive = ClientKeepAlive::new(
            self.client_pool.clone(),
            self.connection_manager.clone(),
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            raw_stop_send.clone(),
        );
        self.daemon_runtime.spawn(async move {
            keep_alive.start_heartbeat_check().await;
        });

        // sync auth info
        let auth_driver = self.auth_driver.clone();
        let raw_stop_send = stop_send.clone();
        self.daemon_runtime.spawn(async move {
            sync_auth_storage_info(auth_driver.clone(), raw_stop_send.clone());
        });

        // flapping detect
        let update_flapping_detect_cache =
            UpdateFlappingDetectCache::new(stop_send.clone(), self.cache_manager.clone());
        self.daemon_runtime.spawn(async move {
            update_flapping_detect_cache.start_update().await;
        });

        // observability
        let cache_manager = self.cache_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let client_pool = self.client_pool.clone();
        let raw_stop_send = stop_send.clone();
        self.daemon_runtime.spawn(async move {
            start_observability(
                cache_manager,
                message_storage_adapter,
                client_pool,
                raw_stop_send,
            )
            .await;
        });

        // metrics record
        let metrics_cache_manager = self.metrics_cache_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let connection_manager = self.connection_manager.clone();
        let raw_stop_send = stop_send.clone();
        self.daemon_runtime.spawn(async move {
            metrics_record_thread(
                metrics_cache_manager.clone(),
                cache_manager.clone(),
                subscribe_manager.clone(),
                connection_manager.clone(),
                60,
                raw_stop_send.clone(),
            );

            metrics_gc_thread(metrics_cache_manager.clone(), raw_stop_send.clone());
        });
    }

    fn start_server(&self, stop_send: broadcast::Sender<bool>) {
        // grpc server
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
            self.metrics_cache_manager.clone(),
        );
        self.admin_runtime.spawn(async move {
            if let Err(e) = server.start().await {
                panic!("{}", e.to_string());
            }
        });

        // prometheus server
        if conf.prometheus.enable {
            self.admin_runtime.spawn(async move {
                register_prometheus_export(conf.prometheus.port).await;
            });
        }

        // pprof server
        let conf = broker_mqtt_conf();
        if conf.pprof.enable {
            self.admin_runtime.spawn(async move {
                start_pprof_monitor(conf.pprof.port, conf.pprof.frequency).await;
            });
        }

        // mqtt tcp server
        let server = self.server.clone();
        self.server_runtime.spawn(async move {
            if let Err(e) = server.start().await {
                panic!("{}", e);
            }
        });

        // mqtt quic server
        let cache = self.cache_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let client_pool = self.client_pool.clone();
        let connection_manager = self.connection_manager.clone();
        let auth_driver = self.auth_driver.clone();
        let delay_message_manager = self.delay_message_manager.clone();
        let schema_manager = self.schema_manager.clone();
        let raw_stop_send = stop_send.clone();
        self.server_runtime.spawn(async move {
            start_quic_server(
                subscribe_manager,
                cache,
                connection_manager,
                message_storage_adapter,
                delay_message_manager,
                client_pool,
                raw_stop_send,
                auth_driver,
                schema_manager,
            )
            .await
        });

        // websocket server
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
        self.server_runtime
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

        self.server_runtime
            .spawn(async move { websockets_server(ws_state).await });
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

    fn start_init(&self) {
        self.daemon_runtime.block_on(async move {
            if let Err(e) = check_placement_center_status(self.client_pool.clone()).await {
                panic!("{}", e);
            }

            if let Err(e) = init_system_user(&self.cache_manager, &self.client_pool).await {
                panic!("{}", e);
            }

            if let Err(e) = load_metadata_cache(
                &self.cache_manager,
                &self.client_pool,
                &self.auth_driver,
                &self.connector_manager,
                &self.schema_manager,
            )
            .await
            {
                panic!("{}", e);
            }

            let config = broker_mqtt_conf();
            info!("config:");
            match serde_json::to_string_pretty(config) {
                Ok(data) => info!("{}", data),
                Err(e) => {
                    panic!("{}", e);
                }
            };
        });
    }

    pub fn awaiting_stop(&self, stop_send: broadcast::Sender<bool>) {
        self.daemon_runtime.spawn(async move {
            sleep(Duration::from_millis(10)).await;
            info!("MQTT Broker service started successfully...");
        });

        // Wait for the stop signal
        self.daemon_runtime.block_on(async move {
            // register node
            let config = broker_mqtt_conf();
            match register_node(&self.client_pool, &self.cache_manager).await {
                Ok(()) => {
                    let client_pool = self.client_pool.clone();
                    let cache_manager = self.cache_manager.clone();

                    // heartbeat report
                    let raw_stop_send = stop_send.clone();
                    self.daemon_runtime.spawn(async move {
                        let conf = broker_mqtt_conf();
                        report_heartbeat(
                            &client_pool,
                            &cache_manager,
                            &conf.system.heartbeat_timeout,
                            raw_stop_send.clone(),
                        )
                        .await;
                    });

                    info!("Node {} has been successfully registered", config.broker_id);
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
            // Wait for all the request packets in the TCP Channel to be processed completely before starting to stop other processing threads.
            signal::ctrl_c().await.expect("failed to listen for event");
            info!(
                "{}",
                "When ctrl + c is received, the service starts to stop"
            );
            // Stop the Server first, indicating that it will no longer receive request packets.
            self.server.stop().await;
            match stop_send.send(true) {
                Ok(_) => {
                    info!("Process stop signal was sent successfully.");
                    if let Err(e) = self.stop_server().await {
                        error!("{}", e);
                    }
                    info!("Service has been stopped successfully. Exiting the process.");
                }
                Err(_) => {
                    error!("Failed to send stop signal");
                }
            }
        });

        // todo tokio runtime shutdown
    }

    async fn stop_server(&self) -> ResultMqttBrokerError {
        let cluster_storage = ClusterStorage::new(self.client_pool.clone());
        let config = broker_mqtt_conf();
        let _ = self.delay_message_manager.stop().await;
        cluster_storage.unregister_node(config).await?;
        info!(
            "Node {} has been successfully unregistered",
            config.broker_id
        );
        self.connection_manager.close_all_connect().await;
        info!("All TCP, TLS, WS, and WSS network connections have been successfully closed.");
        Ok(())
    }
}
