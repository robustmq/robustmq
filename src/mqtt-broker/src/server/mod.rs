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

use crate::core::command::create_command;
use crate::core::event::EventReportManager;

use crate::core::tool::ResultMqttBrokerError;
use crate::storage::session::SessionBatcher;
use crate::{
    core::{cache::MQTTCacheManager, command::CommandContext},
    subscribe::manager::SubscribeManager,
};
use broker_core::cache::NodeCacheManager;
use common_base::task::TaskSupervisor;
use common_config::broker::broker_config;
use common_security::manager::SecurityManager;
use delay_message::manager::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnectionType;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use network_server::context::ServerContext;
use network_server::quic::server::QuicServer;
use network_server::tcp::server::TcpServer;
use network_server::websocket::server::{WebSocketServer, WebSocketServerState};
use node_call::NodeCallManager;
use protocol::robust::RobustMQProtocol;
use rate_limit::global::GlobalRateLimiterManager;
use rate_limit::mqtt::MQTTRateLimiterManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::error;

pub struct Server {
    tcp_server: TcpServer,
    tls_server: TcpServer,
    ws_server: WebSocketServer,
    quic_server: QuicServer,
}

#[derive(Clone)]
pub struct TcpServerContext {
    pub subscribe_manager: Arc<SubscribeManager>,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub client_pool: Arc<ClientPool>,
    pub session_batcher: Arc<SessionBatcher>,
    pub stop_sx: broadcast::Sender<bool>,
    pub security_manager: Arc<SecurityManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub mqtt_limit_manager: Arc<MQTTRateLimiterManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub node_call: Arc<NodeCallManager>,
    pub event_manager: Arc<EventReportManager>,
}

impl Server {
    pub fn new(
        context: TcpServerContext,
        request_channel: Arc<RequestChannel>,
    ) -> (Self, network_server::command::ArcCommandAdapter) {
        let conf = broker_config();
        let command_context = CommandContext {
            cache_manager: context.cache_manager.clone(),
            storage_driver_manager: context.storage_driver_manager.clone(),
            delay_message_manager: context.delay_message_manager.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            client_pool: context.client_pool.clone(),
            session_batcher: context.session_batcher.clone(),
            connection_manager: context.connection_manager.clone(),
            schema_manager: context.schema_manager.clone(),
            security_manager: context.security_manager.clone(),
            rocksdb_engine_handler: context.rocksdb_engine_handler.clone(),
            broker_cache: context.broker_cache.clone(),
            mqtt_limit_manager: context.mqtt_limit_manager.clone(),
            global_limit_manager: context.global_limit_manager.clone(),
            node_call: context.node_call.clone(),
            event_manager: context.event_manager.clone(),
            stop_sx: context.stop_sx.clone(),
        };

        let command = create_command(command_context);
        let mut server_context = ServerContext {
            connection_manager: context.connection_manager.clone(),
            client_pool: context.client_pool.clone(),
            network_type: NetworkConnectionType::Tcp,
            stop_sx: context.stop_sx.clone(),
            broker_cache: context.broker_cache.clone(),
            request_channel: request_channel.clone(),
            global_limit_manager: context.global_limit_manager.clone(),
            task_supervisor: context.task_supervisor.clone(),
        };

        let name = "MQTT".to_string();
        let tcp_server = TcpServer::new(RobustMQProtocol::MQTT4, server_context.clone());

        server_context.network_type = NetworkConnectionType::Tls;
        let tls_server = TcpServer::new(RobustMQProtocol::MQTT4, server_context.clone());

        let ws_server = WebSocketServer::new(WebSocketServerState {
            ws_port: conf.mqtt_server.websocket_port,
            wss_port: conf.mqtt_server.websockets_port,
            connection_manager: context.connection_manager.clone(),
            node_cache: context.broker_cache.clone(),
            global_limit_manager: context.global_limit_manager.clone(),
            stop_sx: context.stop_sx.clone(),
            request_channel: request_channel.clone(),
            protocol: RobustMQProtocol::MQTT4,
        });

        server_context.network_type = NetworkConnectionType::QUIC;
        let quic_server = QuicServer::new(name.clone(), server_context);

        let server = Server {
            tcp_server,
            tls_server,
            ws_server,
            quic_server,
        };
        (server, command)
    }

    pub async fn start(&self) -> ResultMqttBrokerError {
        let conf = broker_config();

        self.tcp_server
            .start(false, conf.mqtt_server.tcp_port)
            .await?;

        self.tls_server
            .start(true, conf.mqtt_server.tls_port)
            .await?;

        let ws_server = self.ws_server.clone();
        tokio::spawn(Box::pin(async move {
            if let Err(e) = ws_server.start_ws().await {
                error!("MQTT WebSocket server start fail, error:{}", e);
                std::process::exit(1);
            }
        }));

        let ws_server = self.ws_server.clone();
        tokio::spawn(Box::pin(async move {
            if let Err(e) = ws_server.start_wss().await {
                error!("MQTT WebSockets server start fail, error:{}", e);
                std::process::exit(1);
            }
        }));

        self.quic_server.start(conf.mqtt_server.quic_port).await?;
        Ok(())
    }

    pub async fn stop(&self) {
        self.tcp_server.stop().await;
        self.tls_server.stop().await;
        self.quic_server.stop().await;
    }
}
