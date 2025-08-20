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

use crate::handler::cache::CacheManager;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use system_topic::SystemTopic;
use tokio::sync::broadcast;

pub mod slow;
pub mod system_topic;

pub async fn start_observability(
    cache_manager: Arc<CacheManager>,
    message_storage_adapter: ArcStorageAdapter,
    client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    let system_topic = SystemTopic::new(
        cache_manager.clone(),
        message_storage_adapter.clone(),
        client_pool.clone(),
    );

    system_topic.start_thread(stop_send).await;
}
