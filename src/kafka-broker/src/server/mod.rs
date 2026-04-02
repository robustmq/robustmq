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
use protocol::robust::RobustMQProtocol;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct KafkaServerParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub stop_sx: broadcast::Sender<bool>,
    pub request_channel: Arc<RequestChannel>,
}

pub struct KafkaServer {
    tcp_server: TcpServer,
}

impl KafkaServer {
    pub fn new(params: KafkaServerParams) -> Self {
        let server_context = ServerContext {
            connection_manager: params.connection_manager.clone(),
            client_pool: params.client_pool,
            network_type: NetworkConnectionType::Tcp,
            stop_sx: params.stop_sx,
            broker_cache: params.broker_cache,
            request_channel: params.request_channel,
            global_limit_manager: params.global_limit_manager,
            task_supervisor: params.task_supervisor,
        };

        let tcp_server = TcpServer::new(RobustMQProtocol::KAFKA, server_context);

        KafkaServer { tcp_server }
    }

    pub async fn start(&self, port: u32) -> ResultCommonError {
        self.tcp_server.start(false, port).await
    }

    pub async fn stop(&self) {
        self.tcp_server.stop().await;
    }
}
