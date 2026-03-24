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

use crate::handler::command::create_command;
use broker_core::cache::NodeCacheManager;
use common_base::error::ResultCommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnectionType;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use network_server::common::handler::handler_process;
use network_server::context::{ProcessorConfig, ServerContext};
use network_server::tcp::server::TcpServer;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct KafkaServer {
    tcp_server: TcpServer,
    handler_process_num: usize,
    connection_manager: Arc<ConnectionManager>,
    command: network_server::command::ArcCommandAdapter,
    request_channel: Arc<RequestChannel>,
    stop_sx: broadcast::Sender<bool>,
}

impl KafkaServer {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        client_pool: Arc<ClientPool>,
        broker_cache: Arc<NodeCacheManager>,
        global_limit_manager: Arc<GlobalRateLimiterManager>,
        stop_sx: broadcast::Sender<bool>,
        proc_config: ProcessorConfig,
    ) -> Self {
        let command = create_command();
        let request_channel = Arc::new(RequestChannel::new(proc_config.channel_size));

        let server_context = ServerContext {
            connection_manager: connection_manager.clone(),
            client_pool,
            command: command.clone(),
            network_type: NetworkConnectionType::Tcp,
            proc_config,
            stop_sx: stop_sx.clone(),
            broker_cache,
            request_channel: request_channel.clone(),
            global_limit_manager,
        };

        let tcp_server = TcpServer::new("KAFKA".to_string(), server_context);

        KafkaServer {
            tcp_server,
            handler_process_num: proc_config.handler_process_num,
            connection_manager,
            command,
            request_channel,
            stop_sx,
        }
    }

    pub async fn start(&self, port: u32) -> ResultCommonError {
        handler_process(
            self.handler_process_num,
            self.connection_manager.clone(),
            self.command.clone(),
            self.request_channel.clone(),
            self.stop_sx.clone(),
        );

        self.tcp_server.start(false, port).await
    }

    pub async fn stop(&self) {
        self.tcp_server.stop().await;
    }
}
