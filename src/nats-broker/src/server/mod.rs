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

use broker_core::cache::NodeCacheManager;
use common_base::error::ResultCommonError;
use common_base::task::TaskSupervisor;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnectionType;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use network_server::context::ServerContext;
use network_server::tcp::server::TcpServer;
use network_server::websocket::server::{WebSocketServer, WebSocketServerState};
use protocol::robust::RobustMQProtocol;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::error;

pub struct NatsServerParams {
    pub tcp_port: u32,
    pub tls_port: u32,
    pub ws_port: u32,
    pub wss_port: u32,
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub stop_sx: broadcast::Sender<bool>,
    pub request_channel: Arc<RequestChannel>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
}

pub struct NatsServer {
    tcp_server: TcpServer,
    tls_server: TcpServer,
    ws_server: WebSocketServer,
    tcp_port: u32,
    tls_port: u32,
}

impl NatsServer {
    pub fn new(params: NatsServerParams) -> Self {
        let mut server_context = ServerContext {
            connection_manager: params.connection_manager.clone(),
            client_pool: params.client_pool,
            network_type: NetworkConnectionType::Tcp,
            stop_sx: params.stop_sx.clone(),
            broker_cache: params.broker_cache.clone(),
            request_channel: params.request_channel.clone(),
            global_limit_manager: params.global_limit_manager.clone(),
            task_supervisor: params.task_supervisor.clone(),
        };

        let tcp_server = TcpServer::new(RobustMQProtocol::NATS, server_context.clone());

        server_context.network_type = NetworkConnectionType::Tls;
        let tls_server = TcpServer::new(RobustMQProtocol::NATS, server_context);

        let ws_server = WebSocketServer::new(WebSocketServerState {
            ws_port: params.ws_port,
            wss_port: params.wss_port,
            connection_manager: params.connection_manager,
            node_cache: params.broker_cache,
            global_limit_manager: params.global_limit_manager,
            stop_sx: params.stop_sx,
            request_channel: params.request_channel,
            protocol: RobustMQProtocol::NATS,
        });

        NatsServer {
            tcp_server,
            tls_server,
            ws_server,
            tcp_port: params.tcp_port,
            tls_port: params.tls_port,
        }
    }

    pub async fn start(&self) -> ResultCommonError {
        self.tcp_server.start(false, self.tcp_port).await?;
        self.tls_server.start(true, self.tls_port).await?;

        let ws = self.ws_server.clone();
        tokio::spawn(Box::pin(async move {
            if let Err(e) = ws.start_ws().await {
                error!("NATS WebSocket server failed to start: {}", e);
                std::process::exit(1);
            }
        }));

        let wss = self.ws_server.clone();
        tokio::spawn(Box::pin(async move {
            if let Err(e) = wss.start_wss().await {
                error!("NATS WebSocket TLS server failed to start: {}", e);
                std::process::exit(1);
            }
        }));

        Ok(())
    }

    pub async fn stop(&self) {
        self.tcp_server.stop().await;
        self.tls_server.stop().await;
    }
}
