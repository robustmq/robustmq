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
use common_base::{role::is_broker_node, task::TaskSupervisor};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use nats_broker::broker::{NatsBrokerServer, NatsBrokerServerParams};
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;

use crate::BrokerServer;

pub fn build_nats_params(
    connection_manager: Arc<ConnectionManager>,
    client_pool: Arc<ClientPool>,
    broker_cache: Arc<NodeCacheManager>,
    global_limit_manager: Arc<GlobalRateLimiterManager>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_sx: broadcast::Sender<bool>,
    shared_request_channel: Arc<RequestChannel>,
    storage_driver_manager: Arc<StorageDriverManager>,
) -> NatsBrokerServerParams {
    let config = broker_config();
    NatsBrokerServerParams {
        connection_manager,
        client_pool,
        broker_cache,
        global_limit_manager,
        task_supervisor,
        stop_sx,
        proc_config: network_server::context::ProcessorConfig {
            accept_thread_num: config.nats_runtime.network.accept_thread_num,
            handler_process_num: 0,
            channel_size: 0,
        },
        request_channel: shared_request_channel,
        storage_driver_manager,
    }
}

impl BrokerServer {
    pub fn start_nats_broker(&self, stop: broadcast::Sender<bool>) {
        if !is_broker_node(&self.config.roles) {
            return;
        }
        let mut params = self.nats_params.clone();
        params.stop_sx = stop;
        let server = NatsBrokerServer::new(params);
        self.broker_runtime.spawn(Box::pin(async move {
            server.start().await;
        }));
    }
}
