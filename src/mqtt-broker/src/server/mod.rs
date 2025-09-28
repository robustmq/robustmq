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

use crate::common::types::ResultMqttBrokerError;
use crate::handler::command::create_command;
use crate::{
    handler::{cache::MQTTCacheManager, command::CommandContext},
    security::AuthDriver,
    subscribe::manager::SubscribeManager,
};
use broker_core::cache::BrokerCacheManager;
use broker_core::rocksdb::RocksDBEngine;
use common_config::broker::broker_config;
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnectionType;
use network_server::common::connection_manager::ConnectionManager;
use network_server::context::{ProcessorConfig, ServerContext};
use network_server::quic::server::QuicServer;
use network_server::tcp::server::TcpServer;
use network_server::websocket::server::{WebSocketServer, WebSocketServerState};
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::sync::broadcast;
use tracing::error;

pub mod inner;

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
    pub message_storage_adapter: ArcStorageAdapter,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub client_pool: Arc<ClientPool>,
    pub stop_sx: broadcast::Sender<bool>,
    pub auth_driver: Arc<AuthDriver>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub broker_cache: Arc<BrokerCacheManager>,
}

impl Server {
    pub fn new(context: TcpServerContext) -> Self {
        let conf = broker_config();
        let command_context = CommandContext {
            cache_manager: context.cache_manager.clone(),
            message_storage_adapter: context.message_storage_adapter.clone(),
            delay_message_manager: context.delay_message_manager.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            client_pool: context.client_pool.clone(),
            connection_manager: context.connection_manager.clone(),
            schema_manager: context.schema_manager.clone(),
            auth_driver: context.auth_driver.clone(),
            rocksdb_engine_handler: context.rocksdb_engine_handler.clone(),
            broker_cache: context.broker_cache.clone(),
        };
        let command = create_command(command_context);

        let proc_config = ProcessorConfig {
            accept_thread_num: conf.network.accept_thread_num,
            handler_process_num: conf.network.handler_thread_num,
            response_process_num: conf.network.response_thread_num,
            channel_size: conf.network.queue_size,
        };

        let mut context = ServerContext {
            connection_manager: context.connection_manager.clone(),
            client_pool: context.client_pool.clone(),
            command: command.clone(),
            network_type: NetworkConnectionType::Tcp,
            proc_config,
            stop_sx: context.stop_sx.clone(),
            broker_cache: context.broker_cache.clone(),
        };

        // TCP Server
        let tcp_server = TcpServer::new(context.clone());

        // Tls Server
        context.network_type = NetworkConnectionType::Tls;
        let tls_server = TcpServer::new(context.clone());

        // Websocket Server
        let ws_server = WebSocketServer::new(WebSocketServerState {
            ws_port: conf.mqtt_server.websocket_port,
            wss_port: conf.mqtt_server.websockets_port,
            command: command.clone(),
            connection_manager: context.connection_manager.clone(),
            stop_sx: context.stop_sx.clone(),
        });

        // QuicServer
        context.network_type = NetworkConnectionType::QUIC;
        let quic_server = QuicServer::new(context);
        Server {
            tcp_server,
            tls_server,
            ws_server,
            quic_server,
        }
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
        tokio::spawn(async move {
            if let Err(e) = ws_server.start_ws().await {
                error!("{}", e);
            }
        });

        let ws_server = self.ws_server.clone();
        tokio::spawn(async move {
            if let Err(e) = ws_server.start_wss().await {
                error!("{}", e);
            }
        });

        self.quic_server.start(conf.mqtt_server.quic_port).await?;
        Ok(())
    }

    pub async fn stop(&self) {
        self.tcp_server.stop().await;
        self.tls_server.stop().await;
        self.quic_server.stop().await;
    }
}
