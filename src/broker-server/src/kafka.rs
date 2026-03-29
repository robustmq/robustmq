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
use kafka_broker::broker::{KafkaBrokerServer, KafkaBrokerServerParams};
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;

use crate::BrokerServer;

pub struct KafkaBuildParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub stop_sx: broadcast::Sender<bool>,
    pub shared_request_channel: Arc<RequestChannel>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
}

pub fn build_kafka_params(p: KafkaBuildParams) -> KafkaBrokerServerParams {
    let config = broker_config();
    KafkaBrokerServerParams {
        connection_manager: p.connection_manager,
        client_pool: p.client_pool,
        broker_cache: p.broker_cache,
        global_limit_manager: p.global_limit_manager,
        task_supervisor: p.task_supervisor,
        stop_sx: p.stop_sx,
        proc_config: network_server::context::ProcessorConfig {
            accept_thread_num: config.kafka_runtime.network.accept_thread_num,
            handler_process_num: 0,
            channel_size: 0,
        },
        request_channel: p.shared_request_channel,
        storage_driver_manager: p.storage_driver_manager,
    }
}

impl BrokerServer {
    pub fn start_kafka_broker(&self, stop: broadcast::Sender<bool>) {
        if !is_broker_node(&self.config.roles) {
            return;
        }
        let mut params = self.kafka_params.clone();
        params.stop_sx = stop;
        let server = KafkaBrokerServer::new(params);
        self.broker_runtime.spawn(Box::pin(async move {
            server.start().await;
        }));
    }
}
