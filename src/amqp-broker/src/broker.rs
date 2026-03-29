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

use crate::server::{AmqpServer, AmqpServerParams};
use broker_core::cache::NodeCacheManager;
use common_base::task::TaskSupervisor;
use grpc_clients::pool::ClientPool;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use network_server::context::ProcessorConfig;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::{error, info};

const DEFAULT_AMQP_PORT: u32 = 5672;

#[derive(Clone)]
pub struct AmqpBrokerServerParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub stop_sx: broadcast::Sender<bool>,
    pub proc_config: ProcessorConfig,
    pub request_channel: Arc<RequestChannel>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
}

pub struct AmqpBrokerServer {
    server: AmqpServer,
    stop_sx: broadcast::Sender<bool>,
}

impl AmqpBrokerServer {
    pub fn new(params: AmqpBrokerServerParams) -> Self {
        let server = AmqpServer::new(AmqpServerParams {
            connection_manager: params.connection_manager,
            client_pool: params.client_pool,
            broker_cache: params.broker_cache,
            global_limit_manager: params.global_limit_manager,
            task_supervisor: params.task_supervisor,
            stop_sx: params.stop_sx.clone(),
            proc_config: params.proc_config,
            request_channel: params.request_channel,
            storage_driver_manager: params.storage_driver_manager,
        });
        AmqpBrokerServer {
            server,
            stop_sx: params.stop_sx,
        }
    }

    pub async fn start(&self) {
        if let Err(e) = self.server.start(DEFAULT_AMQP_PORT).await {
            error!("AMQP broker server failed to start: {}", e);
            std::process::exit(1);
        }
        self.awaiting_stop().await;
    }

    pub async fn stop(&self) {
        self.server.stop().await;
    }

    pub async fn awaiting_stop(&self) {
        let mut recv = self.stop_sx.subscribe();
        match recv.recv().await {
            Ok(_) => {
                info!("AMQP broker has stopped.");
                self.server.stop().await;
                info!("AMQP broker service stopped successfully.");
            }
            Err(e) => {
                error!("AMQP broker stop channel error: {}", e);
            }
        }
    }
}
