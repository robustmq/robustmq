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

use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::adapter::record::Record;
use std::{
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use storage_adapter::storage::{ShardInfo, StorageAdapter};
use tokio::time::Instant;
use tokio_util::time::DelayQueue;

pub mod delay_pop;

#[derive(Clone)]
pub struct DelayMessageRecord {
    shard_name: String,
    offset: u64,
    delay_timestamp: u64,
}

pub struct DelayMessageManager<S> {
    namespace: String,
    shard_num: u32,
    message_storage_adapter: Arc<S>,
    incr_no: AtomicU64,
    delay_queue_list: DashMap<u64, DelayQueue<DelayMessageRecord>>,
}

const DELAY_MESSAGE_SHARD_NAME_PREFIX: &str = "$delay-message-shard-";
impl<S> DelayMessageManager<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(namespace: String, shard_num: u32, message_storage_adapter: Arc<S>) -> Self {
        DelayMessageManager {
            namespace,
            shard_num,
            message_storage_adapter,
            incr_no: AtomicU64::new(0),
            delay_queue_list: DashMap::with_capacity(2),
        }
    }

    pub async fn init(&self) -> Result<(), CommonError> {
        self.try_init_shard().await?;
        self.start_delay_queue().await?;
        println!("DelayMessage service start");
        Ok(())
    }

    async fn try_init_shard(&self) -> Result<(), CommonError> {
        for i in 0..self.shard_num {
            let shard_name = self.get_delay_message_shard_name(i);
            let shard_info = self
                .message_storage_adapter
                .get_shard(self.namespace.clone(), shard_name.clone())
                .await?;
            if shard_info.is_none() {
                let shard = ShardInfo {
                    namespace: self.namespace.clone(),
                    shard_name: shard_name.clone(),
                    replica_num: 1,
                };
                self.message_storage_adapter.create_shard(shard).await?;
                println!("init shard:{}, {}", self.namespace.clone(), shard_name);
            }
        }

        Ok(())
    }

    pub async fn send_delay_message(&self, data: Record) -> Result<(), CommonError> {
        let shard_no = self.get_target_shard_no();
        let namespace = self.namespace.clone();
        let shard_name = self.get_delay_message_shard_name(shard_no as u32);
        let offset = self
            .message_storage_adapter
            .write(namespace, shard_name.clone(), data.clone())
            .await?;

        let delay_message_record = DelayMessageRecord {
            shard_name,
            offset,
            delay_timestamp: data.delay_timestamp,
        };
        self.send_to_delay_queue(shard_no, delay_message_record)
            .await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), CommonError> {
        Ok(())
    }

    async fn send_to_delay_queue(
        &self,
        shard_no: u64,
        delay_message_record: DelayMessageRecord,
    ) -> Result<(), CommonError> {
        if let Some(mut delay_queue) = self.delay_queue_list.get_mut(&shard_no) {
            delay_queue.insert_at(
                delay_message_record.clone(),
                Instant::now() + Duration::from_secs(delay_message_record.delay_timestamp),
            );
        }
        Ok(())
    }

    async fn start_delay_queue(&self) -> Result<(), CommonError> {
        Ok(())
    }

    fn get_target_shard_no(&self) -> u64 {
        self.incr_no
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.shard_num as u64
    }

    fn get_delay_message_shard_name(&self, no: u32) -> String {
        format!("{}{}", DELAY_MESSAGE_SHARD_NAME_PREFIX, no)
    }
}
