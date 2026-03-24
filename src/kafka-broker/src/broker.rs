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

use crate::server::KafkaServer;
use broker_core::cache::NodeCacheManager;
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use network_server::context::ProcessorConfig;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::error;

const DEFAULT_KAFKA_PORT: u32 = 9095;

#[derive(Clone)]
pub struct KafkaBrokerServerParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub stop_sx: broadcast::Sender<bool>,
    pub proc_config: ProcessorConfig,
}

pub struct KafkaBrokerServer {
    server: KafkaServer,
}

impl KafkaBrokerServer {
    pub fn new(params: KafkaBrokerServerParams) -> Self {
        let server = KafkaServer::new(
            params.connection_manager,
            params.client_pool,
            params.broker_cache,
            params.global_limit_manager,
            params.stop_sx,
            params.proc_config,
        );
        KafkaBrokerServer { server }
    }

    pub async fn start(&self) {
        if let Err(e) = self.server.start(DEFAULT_KAFKA_PORT).await {
            error!("Kafka broker server failed to start: {}", e);
            std::process::exit(1);
        }
    }

    pub async fn stop(&self) {
        self.server.stop().await;
    }
}
