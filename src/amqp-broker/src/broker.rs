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

use crate::server::AmqpServer;
use broker_core::cache::NodeCacheManager;
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use network_server::context::ProcessorConfig;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::error;

const DEFAULT_AMQP_PORT: u32 = 5672;

pub struct AmqpBrokerServer {
    server: AmqpServer,
}

impl AmqpBrokerServer {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        client_pool: Arc<ClientPool>,
        broker_cache: Arc<NodeCacheManager>,
        global_limit_manager: Arc<GlobalRateLimiterManager>,
        stop_sx: broadcast::Sender<bool>,
        proc_config: ProcessorConfig,
    ) -> Self {
        let server = AmqpServer::new(
            connection_manager,
            client_pool,
            broker_cache,
            global_limit_manager,
            stop_sx,
            proc_config,
        );
        AmqpBrokerServer { server }
    }

    pub async fn start(&self) {
        if let Err(e) = self.server.start(DEFAULT_AMQP_PORT).await {
            error!("AMQP broker server failed to start: {}", e);
            std::process::exit(1);
        }
    }

    pub async fn stop(&self) {
        self.server.stop().await;
    }
}
