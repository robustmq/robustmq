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
use crate::handler::cache::MQTTCacheManager;
use crate::handler::dynamic_cache::load_metadata_cache;
use crate::handler::flapping_detect::clean_flapping_detect;
use crate::handler::keep_alive::ClientKeepAlive;
use crate::handler::sub_parse_topic::start_parse_subscribe_by_new_topic_thread;
use crate::handler::system_alarm::SystemAlarm;
use crate::security::auth::super_user::init_system_user;
use crate::security::storage::sync::sync_auth_storage_info;
use crate::security::AuthDriver;
use crate::server::{Server, TcpServerContext};
use crate::subscribe::exclusive::ExclusivePush;
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::share::follower::ShareFollowerResub;
use crate::subscribe::share::leader::ShareLeaderPush;
use crate::system_topic::SystemTopic;
use broker_core::cache::BrokerCacheManager;
use broker_core::rocksdb::RocksDBEngine;
use common_config::broker::broker_config;
use delay_message::{start_delay_message_manager, DelayMessageManager};
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::sync::broadcast::{self};
use tracing::{error, info};

#[derive(Clone)]
pub struct MqttBrokerServerParams {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub message_storage_adapter: ArcStorageAdapter,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub connector_manager: Arc<ConnectorManager>,
    pub auth_driver: Arc<AuthDriver>,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub metrics_cache_manager: Arc<MetricsCacheManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub broker_cache: Arc<BrokerCacheManager>,
}

pub struct MqttBrokerServer {
    cache_manager: Arc<MQTTCacheManager>,
    client_pool: Arc<ClientPool>,
    message_storage_adapter: ArcStorageAdapter,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    connector_manager: Arc<ConnectorManager>,
    auth_driver: Arc<AuthDriver>,
    delay_message_manager: Arc<DelayMessageManager>,
    schema_manager: Arc<SchemaRegisterManager>,
    metrics_cache_manager: Arc<MetricsCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    server: Arc<Server>,
    main_stop: broadcast::Sender<bool>,
    inner_stop: broadcast::Sender<bool>,
}

impl MqttBrokerServer {
    pub fn new(params: MqttBrokerServerParams, main_stop: broadcast::Sender<bool>) -> Self {
        let (inner_stop, _) = broadcast::channel(2);
        let server = Arc::new(Server::new(TcpServerContext {
            subscribe_manager: params.subscribe_manager.clone(),
            cache_manager: params.cache_manager.clone(),
            connection_manager: params.connection_manager.clone(),
            message_storage_adapter: params.message_storage_adapter.clone(),
            delay_message_manager: params.delay_message_manager.clone(),
            schema_manager: params.schema_manager.clone(),
            client_pool: params.client_pool.clone(),
            stop_sx: inner_stop.clone(),
            auth_driver: params.auth_driver.clone(),
            rocksdb_engine_handler: params.rocksdb_engine_handler.clone(),
            broker_cache: params.broker_cache.clone(),
        }));

        MqttBrokerServer {
            main_stop,
            inner_stop,
            cache_manager: params.cache_manager,
            client_pool: params.client_pool,
            message_storage_adapter: params.message_storage_adapter,
            subscribe_manager: params.subscribe_manager,
            connector_manager: params.connector_manager,
            connection_manager: params.connection_manager,
            auth_driver: params.auth_driver,
            delay_message_manager: params.delay_message_manager,
            schema_manager: params.schema_manager,
            server,
            metrics_cache_manager: params.metrics_cache_manager,
            rocksdb_engine_handler: params.rocksdb_engine_handler,
        }
    }

    pub async fn start(&self) {
        self.start_init().await;

        self.start_daemon_thread();

        self.start_delay_message_thread();

        self.start_connector_thread();

        self.start_subscribe_push();

        self.start_server();

        self.awaiting_stop().await;
    }

    fn start_daemon_thread(&self) {
        // client keep alive
        let raw_stop_send = self.inner_stop.clone();
        let keep_alive = ClientKeepAlive::new(
            self.client_pool.clone(),
            self.connection_manager.clone(),
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            raw_stop_send.clone(),
        );
        tokio::spawn(async move {
            keep_alive.start_heartbeat_check().await;
        });

        // sync auth info
        let auth_driver = self.auth_driver.clone();
        let raw_stop_send = self.inner_stop.clone();
        tokio::spawn(async move {
            sync_auth_storage_info(auth_driver.clone(), raw_stop_send.clone());
        });

        // flapping detect
        let stop_send = self.inner_stop.clone();
        let cache_manager = self.cache_manager.clone();
        tokio::spawn(async move {
            clean_flapping_detect(cache_manager, stop_send).await;
        });

        // observability
        let raw_stop_send = self.inner_stop.clone();
        let system_topic = SystemTopic::new(
            self.cache_manager.clone(),
            self.message_storage_adapter.clone(),
            self.client_pool.clone(),
        );
        tokio::spawn(async move {
            system_topic.start_thread(raw_stop_send).await;
        });

        // metrics record
        let metrics_cache_manager = self.metrics_cache_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let connection_manager = self.connection_manager.clone();
        let raw_stop_send = self.inner_stop.clone();
        tokio::spawn(async move {
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

        // system alarm
        let system_alarm = SystemAlarm::new(
            self.client_pool.clone(),
            self.cache_manager.clone(),
            self.message_storage_adapter.clone(),
            self.rocksdb_engine_handler.clone(),
            self.inner_stop.clone(),
        );
        tokio::spawn(async move {
            if let Err(e) = system_alarm.start().await {
                error!("system alarm error:{}", e);
            }
        });
    }

    fn start_server(&self) {
        let server = self.server.clone();
        tokio::spawn(async move {
            if let Err(e) = server.start().await {
                panic!("{}", e);
            }
        });
    }

    fn start_connector_thread(&self) {
        let message_storage = self.message_storage_adapter.clone();
        let connector_manager = self.connector_manager.clone();
        let stop_send = self.inner_stop.clone();
        tokio::spawn(async move {
            start_connector_thread(message_storage, connector_manager, stop_send).await;
        });
    }

    fn start_subscribe_push(&self) {
        let subscribe_manager = self.subscribe_manager.clone();
        let client_pool = self.client_pool.clone();
        let metadata_cache = self.cache_manager.clone();
        let stop_send = self.inner_stop.clone();

        tokio::spawn(async move {
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
            self.metrics_cache_manager.clone(),
            self.rocksdb_engine_handler.clone(),
        );

        tokio::spawn(async move {
            exclusive_sub.start().await;
        });

        let leader_sub = ShareLeaderPush::new(
            self.subscribe_manager.clone(),
            self.message_storage_adapter.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            self.rocksdb_engine_handler.clone(),
        );

        tokio::spawn(async move {
            leader_sub.start().await;
        });

        let follower_sub = ShareFollowerResub::new(
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            self.client_pool.clone(),
        );

        tokio::spawn(async move {
            follower_sub.start().await;
        });
    }

    fn start_delay_message_thread(&self) {
        let delay_message_manager = self.delay_message_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        tokio::spawn(async move {
            let conf = broker_config();
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

    async fn start_init(&self) {
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
    }

    pub async fn awaiting_stop(&self) {
        // Wait for the stop signal
        let server = self.server.clone();
        let delay_message_manager = self.delay_message_manager.clone();
        let connection_manager = self.connection_manager.clone();
        let mut recv = self.main_stop.subscribe();
        let raw_inner_stop = self.inner_stop.clone();
        // Stop the Server first, indicating that it will no longer receive request packets.
        match recv.recv().await {
            Ok(_) => {
                info!("Broker has stopped.");
                server.stop().await;
                match raw_inner_stop.send(true) {
                    Ok(_) => {
                        info!("Process stop signal was sent successfully.");
                        if let Err(e) = MqttBrokerServer::stop_server(
                            &delay_message_manager,
                            &connection_manager,
                        )
                        .await
                        {
                            error!("{}", e);
                        }
                        info!("Service has been stopped successfully. Exiting the process.");
                    }
                    Err(_) => {
                        error!("Failed to send stop signal");
                    }
                }
            }
            Err(e) => {
                error!("recv error {}", e);
            }
        }
    }

    async fn stop_server(
        delay_message_manager: &Arc<DelayMessageManager>,
        connection_manager: &Arc<ConnectionManager>,
    ) -> ResultMqttBrokerError {
        let _ = delay_message_manager.stop().await;
        connection_manager.close_all_connect().await;
        info!("All TCP, TLS, WS, and WSS network connections have been successfully closed.");
        Ok(())
    }
}
