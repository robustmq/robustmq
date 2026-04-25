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

use common_base::task::TaskSupervisor;
pub use manager::NatsSubscribeManager;
use network_server::common::connection_manager::ConnectionManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;

use crate::{
    core::cache::NatsCacheManager,
    push::{parse::start_subscribe_parse_thread, thread::start_sub_push_thread},
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

#[allow(clippy::too_many_arguments)]
pub async fn start_sub_task(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    cache_manager: Arc<NatsCacheManager>,
    client_pool: Arc<grpc_clients::pool::ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    task_supervisor: Arc<TaskSupervisor>,
    push_thread_num: usize,
    stop_sx: broadcast::Sender<bool>,
) {
    // parse thread
    start_subscribe_parse_thread(subscribe_manager, cache_manager, &task_supervisor, &stop_sx)
        .await;

    // push thread
    start_sub_push_thread(
        subscribe_manager,
        client_pool,
        connection_manager,
        storage_driver_manager,
        task_supervisor,
        push_thread_num,
        stop_sx,
    )
    .await;
}
