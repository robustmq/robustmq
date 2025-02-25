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

use std::sync::Arc;

use crate::DelayMessageManager;
use futures::StreamExt;
use storage_adapter::storage::StorageAdapter;

pub async fn pop_delay_queue<S>(delay_message_manager: Arc<DelayMessageManager<S>>, shard_no: u64)
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    if let Some(mut delay_queue) = delay_message_manager.delay_queue_list.get_mut(&shard_no) {
        while let Some(expired) = delay_queue.next().await {
            let record = expired.into_inner();
            println!("Expired item: {},{}", record.shard_name, record.offset);
        }
    }
}
