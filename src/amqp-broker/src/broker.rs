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

use crate::core::cache::AmqpCacheManager;
use crate::core::keep_alive::AmqpKeepAlive;
use crate::core::recovery::AmqpRecoveryScanner;
use crate::server::{AmqpServer, AmqpServerParams};
use broker_core::cache::NodeCacheManager;
use common_base::task::TaskSupervisor;
use common_config::broker::broker_config;
use common_security::manager::SecurityManager;
use grpc_clients::pool::ClientPool;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::{error, info};

#[derive(Clone)]
pub struct AmqpBrokerServerParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub stop_sx: broadcast::Sender<bool>,
    pub request_channel: Arc<RequestChannel>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub amqp_cache: Arc<AmqpCacheManager>,
    pub security_manager: Arc<SecurityManager>,
}

pub struct AmqpBrokerServer {
    server: AmqpServer,
    stop_sx: broadcast::Sender<bool>,
    keep_alive: AmqpKeepAlive,
    recovery_scanner: AmqpRecoveryScanner,
}

impl AmqpBrokerServer {
    pub fn new(params: AmqpBrokerServerParams) -> Self {
        let keep_alive =
            AmqpKeepAlive::new(params.connection_manager.clone(), params.amqp_cache.clone());
        let recovery_scanner = AmqpRecoveryScanner::new(
            params.client_pool.clone(),
            params.storage_driver_manager.clone(),
        );

        let server = AmqpServer::new(AmqpServerParams {
            connection_manager: params.connection_manager,
            client_pool: params.client_pool,
            broker_cache: params.broker_cache,
            global_limit_manager: params.global_limit_manager,
            task_supervisor: params.task_supervisor,
            stop_sx: params.stop_sx.clone(),
            request_channel: params.request_channel,
            storage_driver_manager: params.storage_driver_manager,
        });
        AmqpBrokerServer {
            server,
            stop_sx: params.stop_sx,
            keep_alive,
            recovery_scanner,
        }
    }

    pub async fn start(&self) -> Result<(), std::io::Error> {
        let keep_alive = self.keep_alive.clone();
        let keep_alive_stop = self.stop_sx.clone();
        tokio::spawn(async move { keep_alive.start(&keep_alive_stop).await });

        let recovery_scanner = self.recovery_scanner.clone();
        let recovery_stop = self.stop_sx.clone();
        tokio::spawn(async move { recovery_scanner.start(&recovery_stop).await });

        let port = broker_config().amqp_runtime.tcp_port;
        self.server.start(port).await.map_err(|e| {
            std::io::Error::other(format!(
                "AMQP broker server failed to start on port {}: {}",
                port, e
            ))
        })?;
        self.awaiting_stop().await;
        Ok(())
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
