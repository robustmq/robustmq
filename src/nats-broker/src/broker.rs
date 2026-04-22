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
    push::{manager::NatsSubscribeManager, start_push},
    server::{NatsServer, NatsServerParams},
};
use broker_core::cache::NodeCacheManager;
use common_base::task::{TaskKind, TaskSupervisor};
use common_config::broker::broker_config;
use common_security::manager::SecurityManager;
use grpc_clients::pool::ClientPool;
use mq9_core::public::try_init_system_email;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
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
    client_pool: Arc<ClientPool>,
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
            client_pool: params.client_pool.clone(),
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
            params.client_pool.clone(),
        );
        NatsBrokerServer {
            server,
            keep_alive,
            cache_manager: params.cache_manager,
            subscribe_manager: params.subscribe_manager,
            connection_manager: params.connection_manager,
            storage_driver_manager: params.storage_driver_manager,
            client_pool: params.client_pool,
            task_supervisor: params.task_supervisor,
            stop_sx: params.stop_sx,
        }
    }

    pub async fn start(&self) {
        let conf = broker_config();

        if let Err(e) = try_init_system_email(
            &self.cache_manager.node_cache,
            &self.storage_driver_manager,
            &self.client_pool,
        )
        .await
        {
            error!("Failed to init system mailbox: {}", e);
            std::process::exit(1);
        }

        start_push(
            &self.subscribe_manager,
            self.cache_manager.clone(),
            self.client_pool.clone(),
            self.connection_manager.clone(),
            self.storage_driver_manager.clone(),
            self.task_supervisor.clone(),
            conf.nats_runtime.push_thread_num,
            self.stop_sx.clone(),
        )
        .await;

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
