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

use crate::{
    core::{cache::NatsCacheManager, keep_alive::NatsClientKeepAlive},
    server::{NatsServer, NatsServerParams},
    subscribe::{
        directly_push::DirectlyPushManager, parse::start_parse_thread,
        queue_push::QueuePushManager, NatsSubscribeManager,
    },
};
use broker_core::cache::NodeCacheManager;
use common_base::task::{TaskKind, TaskSupervisor};
use common_config::broker::broker_config;
use common_security::manager::SecurityManager;
use grpc_clients::pool::ClientPool;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

#[derive(Clone)]
pub struct NatsBrokerServerParams {
    pub cache_manager: Arc<NatsCacheManager>,
    pub subscribe_manager: Arc<NatsSubscribeManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub stop_sx: broadcast::Sender<bool>,
    pub request_channel: Arc<RequestChannel>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub security_manager: Arc<SecurityManager>,
}

pub struct NatsBrokerServer {
    server: NatsServer,
    keep_alive: NatsClientKeepAlive,
    cache_manager: Arc<NatsCacheManager>,
    subscribe_manager: Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_sx: broadcast::Sender<bool>,
}

impl NatsBrokerServer {
    pub fn new(params: NatsBrokerServerParams) -> Self {
        let conf = broker_config();
        let server = NatsServer::new(NatsServerParams {
            tcp_port: conf.nats_runtime.tcp_port,
            tls_port: conf.nats_runtime.tls_port,
            ws_port: conf.nats_runtime.ws_port,
            wss_port: conf.nats_runtime.wss_port,
            connection_manager: params.connection_manager.clone(),
            client_pool: params.client_pool,
            broker_cache: params.broker_cache,
            global_limit_manager: params.global_limit_manager,
            task_supervisor: params.task_supervisor.clone(),
            stop_sx: params.stop_sx.clone(),
            request_channel: params.request_channel,
            storage_driver_manager: params.storage_driver_manager.clone(),
            subscribe_manager: params.subscribe_manager.clone(),
            security_manager: params.security_manager,
        });
        let keep_alive = NatsClientKeepAlive::new(
            params.connection_manager.clone(),
            params.cache_manager.clone(),
            params.subscribe_manager.clone(),
        );
        NatsBrokerServer {
            server,
            keep_alive,
            cache_manager: params.cache_manager,
            subscribe_manager: params.subscribe_manager,
            connection_manager: params.connection_manager,
            storage_driver_manager: params.storage_driver_manager,
            task_supervisor: params.task_supervisor,
            stop_sx: params.stop_sx,
        }
    }

    pub async fn start(&self) {
        // Start parse thread
        let (parse_tx, parse_rx) = mpsc::channel(1024);
        self.subscribe_manager.set_parse_sender(parse_tx).await;

        let cache_manager = self.cache_manager.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let stop_sx = self.stop_sx.clone();
        self.task_supervisor
            .spawn(TaskKind::NATSSubscribeParse.to_string(), async move {
                start_parse_thread(cache_manager, subscribe_manager, parse_rx, stop_sx).await;
            });

        // Start directly push manager
        let directly_push = DirectlyPushManager::new(
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.storage_driver_manager.clone(),
        );
        let stop_sx = self.stop_sx.clone();
        self.task_supervisor
            .spawn(TaskKind::NATSSubscribePush.to_string(), async move {
                directly_push.start(&stop_sx).await;
            });

        // Start queue push manager
        let queue_push = QueuePushManager::new(
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.storage_driver_manager.clone(),
        );
        let stop_sx = self.stop_sx.clone();
        self.task_supervisor
            .spawn(TaskKind::NATSQueuePush.to_string(), async move {
                queue_push.start(&stop_sx).await;
            });

        // Start keep-alive monitor
        let keep_alive = self.keep_alive.clone();
        let stop_sx = self.stop_sx.clone();
        self.task_supervisor
            .spawn(TaskKind::NATSClientKeepAlive.to_string(), async move {
                keep_alive.start_heartbeat_check(&stop_sx).await;
            });

        if let Err(e) = self.server.start().await {
            error!("NATS broker server failed to start: {}", e);
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
                info!("NATS broker has stopped.");
                self.server.stop().await;
                info!("NATS broker service stopped successfully.");
            }
            Err(e) => {
                error!("NATS broker stop channel error: {}", e);
            }
        }
    }
}
