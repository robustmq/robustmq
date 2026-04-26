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
use common_base::task::TaskSupervisor;
use grpc_clients::pool::ClientPool;
pub use manager::NatsSubscribeManager;
use network_server::common::connection_manager::ConnectionManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;

use crate::{
    core::cache::NatsCacheManager,
    push::{
        parse::start_subscribe_parse_thread,
        thread::{start_sub_push_thread, SubPushThreadParams},
    },
};
pub mod buckets;
pub mod common;
pub mod manager;
pub mod mq9_fanout;
pub mod mq9_queue;
pub mod nats_fanout;
pub mod nats_queue;
pub mod parse;
pub mod thread;

pub struct SubTaskParams {
    pub cache_manager: Arc<NatsCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub node_cache: Arc<NodeCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub push_thread_num: usize,
    pub stop_sx: broadcast::Sender<bool>,
}

pub async fn start_sub_task(subscribe_manager: &Arc<NatsSubscribeManager>, p: SubTaskParams) {
    start_subscribe_parse_thread(
        subscribe_manager,
        p.cache_manager,
        &p.task_supervisor,
        &p.stop_sx,
    )
    .await;

    start_sub_push_thread(
        subscribe_manager,
        SubPushThreadParams {
            connection_manager: p.connection_manager,
            storage_driver_manager: p.storage_driver_manager,
            node_cache: p.node_cache,
            client_pool: p.client_pool,
            task_supervisor: p.task_supervisor,
            push_thread_num: p.push_thread_num,
            stop_sx: p.stop_sx,
        },
    )
    .await;
}
