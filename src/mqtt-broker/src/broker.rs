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
use crate::core::cache::MQTTCacheManager;
use crate::core::event::EventReportManager;
use crate::core::flapping_detect::clean_flapping_detect;
use crate::core::keep_alive::ClientKeepAlive;
use crate::core::metrics_cache::metrics_record_thread;
use crate::core::pkid_manager::clean_pkid_data;
use crate::core::qos::init_qos2_inner_topic;
use crate::core::retain::RetainMessageManager;
use crate::core::system_alarm::SystemAlarm;
use crate::core::tool::ResultMqttBrokerError;
use crate::core::topic_rewrite::start_topic_rewrite_convert_thread;
use crate::security::login::super_user::try_init_system_user;
use crate::security::storage::sync::start_auth_sync_thread;
use crate::security::AuthManager;
use crate::server::{Server, TcpServerContext};
use crate::storage::session::SessionBatcher;
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::parse::{start_update_parse_thread, ParseSubscribeData};
use crate::subscribe::PushManager;
use crate::system_topic::SystemTopic;
use broker_core::cache::NodeCacheManager;
use broker_core::tenant::try_init_default_tenant;
use common_base::task::{TaskKind, TaskSupervisor};
use common_config::broker::broker_config;
use connector::manager::ConnectorManager;
use delay_message::manager::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use node_call::NodeCallManager;
use rate_limit::global::GlobalRateLimiterManager;
use rate_limit::mqtt::MQTTRateLimiterManager;
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;
use rocksdb_engine::rocksdb::RocksDBEngine;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use storage_engine::group::OffsetManager;
use tokio::sync::broadcast::{self};
use tokio::sync::mpsc;
use tracing::{error, info};

#[derive(Clone)]
pub struct MqttBrokerServerParams {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub session_batcher: Arc<SessionBatcher>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub connector_manager: Arc<ConnectorManager>,
    pub auth_driver: Arc<AuthManager>,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub metrics_cache_manager: Arc<MQTTMetricsCache>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub node_cache: Arc<NodeCacheManager>,
    pub offset_manager: Arc<OffsetManager>,
    pub retain_message_manager: Arc<RetainMessageManager>,
    pub push_manager: Arc<PushManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub node_call: Arc<NodeCallManager>,
    pub event_manager: Arc<EventReportManager>,
}

pub struct MqttBrokerServer {
    cache_manager: Arc<MQTTCacheManager>,
    client_pool: Arc<ClientPool>,
    session_batcher: Arc<SessionBatcher>,
    event_manager: Arc<EventReportManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    connector_manager: Arc<ConnectorManager>,
    auth_driver: Arc<AuthManager>,
    delay_message_manager: Arc<DelayMessageManager>,
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    offset_manager: Arc<OffsetManager>,
    push_manager: Arc<PushManager>,
    task_supervisor: Arc<TaskSupervisor>,
    server: Arc<Server>,
    stop: broadcast::Sender<bool>,
}

impl MqttBrokerServer {
    pub async fn new(params: MqttBrokerServerParams, stop: broadcast::Sender<bool>) -> Self {
        let limit_config = params.node_cache.get_cluster_config().mqtt_limit;
        let limit_manager = Arc::new(
            match MQTTRateLimiterManager::new(
                params.node_cache.clone(),
                limit_config.cluster.max_publish_rate,
                limit_config.cluster.max_connection_rate,
            ) {
                Ok(data) => data,
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            },
        );

        let server = Arc::new(Server::new(TcpServerContext {
            subscribe_manager: params.subscribe_manager.clone(),
            cache_manager: params.cache_manager.clone(),
            connection_manager: params.connection_manager.clone(),
            storage_driver_manager: params.storage_driver_manager.clone(),
            delay_message_manager: params.delay_message_manager.clone(),
            schema_manager: params.schema_manager.clone(),
            client_pool: params.client_pool.clone(),
            session_batcher: params.session_batcher.clone(),
            stop_sx: stop.clone(),
            auth_driver: params.auth_driver.clone(),
            rocksdb_engine_handler: params.rocksdb_engine_handler.clone(),
            broker_cache: params.node_cache.clone(),
            retain_message_manager: params.retain_message_manager.clone(),
            mqtt_limit_manager: limit_manager.clone(),
            global_limit_manager: params.global_limit_manager.clone(),
            node_call: params.node_call.clone(),
            task_supervisor: params.task_supervisor.clone(),
            event_manager: params.event_manager.clone(),
        }));

        MqttBrokerServer {
            stop,
            cache_manager: params.cache_manager,
            client_pool: params.client_pool,
            session_batcher: params.session_batcher,
            event_manager: params.event_manager,
            storage_driver_manager: params.storage_driver_manager,
            subscribe_manager: params.subscribe_manager,
            connector_manager: params.connector_manager,
            connection_manager: params.connection_manager,
            auth_driver: params.auth_driver,
            delay_message_manager: params.delay_message_manager,
            server,
            metrics_cache_manager: params.metrics_cache_manager,
            rocksdb_engine_handler: params.rocksdb_engine_handler,
            offset_manager: params.offset_manager,
            push_manager: params.push_manager,
            task_supervisor: params.task_supervisor,
        }
    }

    pub async fn start(&self) {
        self.start_daemon_thread().await;

        self.start_subscribe_push().await;

        self.start_server();

        self.awaiting_stop().await;
    }

    async fn start_daemon_thread(&self) {
        // init default tenant
        if let Err(e) =
            try_init_default_tenant(&self.cache_manager.node_cache, &self.client_pool).await
        {
            error!("Failed to initialize default tenant: {}", e);
            std::process::exit(1);
        }

        // init qos inner topic
        if let Err(e) = init_qos2_inner_topic(
            &self.client_pool,
            &self.storage_driver_manager,
            &self.cache_manager.node_cache,
        )
        .await
        {
            error!("Failed to initialize qot inner topic: {}", e);
            std::process::exit(1);
        }

        // init system user
        if let Err(e) = try_init_system_user(&self.cache_manager, &self.client_pool).await {
            error!("Failed to initialize system user: {}", e);
            std::process::exit(1);
        }

        if let Err(e) = self.offset_manager.try_comparison_and_save_offset().await {
            error!("Failed to synchronize offset data: {}", e);
            std::process::exit(1);
        }

        // session batch writer
        let session_batcher = self.session_batcher.clone();
        let client_pool = self.client_pool.clone();
        self.task_supervisor
            .spawn(TaskKind::MQTTSessionBatchSend.to_string(), async move {
                session_batcher.start(client_pool.clone()).await;
            });

        // event report consumer
        let event_manager = self.event_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let storage_driver_manager = self.storage_driver_manager.clone();
        let client_pool = self.client_pool.clone();
        self.task_supervisor
            .spawn(TaskKind::MQTTEventReport.to_string(), async move {
                event_manager
                    .start(cache_manager, storage_driver_manager, client_pool)
                    .await;
            });

        // client keep alive
        let raw_stop_send = self.stop.clone();
        let keep_alive = ClientKeepAlive::new(
            self.client_pool.clone(),
            self.session_batcher.clone(),
            self.connection_manager.clone(),
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
        );
        self.task_supervisor.spawn(
            TaskKind::MQTTClientKeepAlive.to_string(),
            Box::pin(async move {
                keep_alive.start_heartbeat_check(&raw_stop_send).await;
            }),
        );

        // sync auth info
        start_auth_sync_thread(
            self.auth_driver.clone(),
            self.task_supervisor.clone(),
            self.stop.clone(),
        );

        // flapping detect
        let stop_send = self.stop.clone();
        let cache_manager = self.cache_manager.clone();
        self.task_supervisor
            .spawn(TaskKind::MQTTCleanFlappingDetect.to_string(), async move {
                clean_flapping_detect(cache_manager, stop_send).await;
            });

        // clean expired pkid data
        let stop_send = self.stop.clone();
        let cache_manager = self.cache_manager.clone();
        self.task_supervisor
            .spawn(TaskKind::MQTTCleanPkidData.to_string(), async move {
                clean_pkid_data(cache_manager, stop_send).await;
            });

        // report system topic info
        let raw_stop_send = self.stop.clone();
        let system_topic = SystemTopic::new(
            self.cache_manager.clone(),
            self.storage_driver_manager.clone(),
            self.client_pool.clone(),
        );
        self.task_supervisor.spawn(
            TaskKind::MQTTReportSystemTopicData.to_string(),
            Box::pin(async move {
                system_topic.start_thread(raw_stop_send).await;
            }),
        );

        // parse topic rewrite
        let metadata_cache = self.cache_manager.clone();
        let stop_send = self.stop.clone();
        self.task_supervisor
            .spawn(TaskKind::MQTTTopicRewriteConvert.to_string(), async move {
                start_topic_rewrite_convert_thread(metadata_cache, stop_send).await;
            });

        // metrics record
        metrics_record_thread(
            self.metrics_cache_manager.clone(),
            self.cache_manager.clone(),
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.connector_manager.clone(),
            30,
            self.stop.clone(),
            self.task_supervisor.clone(),
        );

        // system alarm
        let system_alarm = SystemAlarm::new(
            self.client_pool.clone(),
            self.cache_manager.clone(),
            self.storage_driver_manager.clone(),
            self.rocksdb_engine_handler.clone(),
        );
        let raw_stop_send = self.stop.clone();
        let config = broker_config();
        if config.mqtt_system_monitor.enable {
            self.task_supervisor
                .spawn(TaskKind::MQTTSystemAlarm.to_string(), async move {
                    if let Err(e) = system_alarm.start(raw_stop_send).await {
                        error!("Failed to start system alarm monitoring: {}", e);
                    }
                });
        }
    }

    fn start_server(&self) {
        let server = self.server.clone();
        tokio::spawn(Box::pin(async move {
            if let Err(e) = server.start().await {
                error!("Failed to start MQTT broker server: {}", e);
                std::process::exit(1);
            }
        }));
    }

    async fn start_subscribe_push(&self) {
        // start push manager
        let stop_send = self.stop.clone();
        let push_manager = self.push_manager.clone();
        self.task_supervisor
            .spawn(TaskKind::MQTTSubscribePush.to_string(), async move {
                push_manager.start(&stop_send).await;
            });

        // parse subscribe data
        let (sx, rx) = mpsc::channel::<ParseSubscribeData>(2000);
        self.subscribe_manager.set_cache_sender(sx).await;
        let subscribe_manager = self.subscribe_manager.clone();
        let client_pool = self.client_pool.clone();
        let cache_manager = self.cache_manager.clone();
        let stop_send = self.stop.clone();
        self.task_supervisor
            .spawn(TaskKind::MQTTSubscribeParse.to_string(), async move {
                start_update_parse_thread(
                    client_pool,
                    cache_manager,
                    subscribe_manager,
                    rx,
                    stop_send,
                )
                .await;
            });
    }

    pub async fn awaiting_stop(&self) {
        // Wait for the stop signal
        let server = self.server.clone();
        let delay_message_manager = self.delay_message_manager.clone();
        let connection_manager = self.connection_manager.clone();
        let mut recv = self.stop.subscribe();
        // Stop the Server first, indicating that it will no longer receive request packets.
        match recv.recv().await {
            Ok(_) => {
                info!("Broker has stopped.");
                server.stop().await;

                info!("Process stop signal was sent successfully.");
                if let Err(e) =
                    MqttBrokerServer::stop_server(&delay_message_manager, &connection_manager).await
                {
                    error!("Failed to stop broker components: {}", e);
                }

                info!("Service has been stopped successfully. Exiting the process.");
            }
            Err(e) => {
                error!("Failed to receive shutdown signal: {}", e);
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
