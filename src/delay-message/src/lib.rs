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

use common_base::{error::common::CommonError, tools::now_second};
use common_metrics::mqtt::statistics::{
    record_mqtt_delay_queue_remaining_capacity_set, record_mqtt_delay_queue_total_capacity_set,
    record_mqtt_delay_queue_used_capacity_set,
};
use dashmap::DashMap;
use delay::{
    get_delay_message_shard_name, init_delay_message_shard, persist_delay_message,
    start_delay_message_pop, start_recover_delay_queue,
};
use metadata_struct::{adapter::record::Record, delay_info::DelayMessageInfo};
use persist::persist_delay_info;
use std::{
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{sync::broadcast, time::Instant};
use tokio_util::time::DelayQueue;

pub mod delay;
pub mod persist;
pub mod pop;

pub async fn start_delay_message_manager(
    delay_message_manager: &Arc<DelayMessageManager>,
    message_storage_adapter: &ArcStorageAdapter,
    namespace: &str,
    shard_num: u64,
) -> Result<(), CommonError> {
    delay_message_manager.start().await;
    init_delay_message_shard(message_storage_adapter, namespace, shard_num).await?;

    start_recover_delay_queue(
        delay_message_manager,
        message_storage_adapter,
        namespace,
        shard_num,
    );

    start_delay_message_pop(
        delay_message_manager,
        message_storage_adapter,
        namespace,
        shard_num,
    );

    Ok(())
}

pub struct DelayMessageManager {
    namespace: String,
    shard_num: u64,
    message_storage_adapter: ArcStorageAdapter,
    incr_no: AtomicU64,
    delay_queue_list: DashMap<u64, DelayQueue<DelayMessageInfo>>,
    delay_queue_pop_thread: DashMap<u64, broadcast::Sender<bool>>,
}

impl DelayMessageManager {
    pub fn new(
        namespace: String,
        shard_num: u64,
        message_storage_adapter: ArcStorageAdapter,
    ) -> Self {
        DelayMessageManager {
            namespace,
            shard_num,
            message_storage_adapter,
            incr_no: AtomicU64::new(0),
            delay_queue_list: DashMap::with_capacity(2),
            delay_queue_pop_thread: DashMap::with_capacity(2),
        }
    }

    pub async fn start(&self) {
        for shard_no in 0..self.shard_num {
            self.delay_queue_list.insert(shard_no, DelayQueue::new());
        }
    }

    pub async fn send(
        &self,
        target_topic: &str,
        delay_timestamp: u64,
        data: Record,
    ) -> Result<(), CommonError> {
        let shard_no = self.get_target_shard_no();
        let namespace = self.namespace.clone();
        let delay_shard_name = get_delay_message_shard_name(shard_no);

        // Persist DelayMessage
        let offset = persist_delay_message(
            &self.message_storage_adapter,
            &namespace,
            &delay_shard_name,
            data,
        )
        .await?;

        // DelayInfo into delay queue
        let delay_info = DelayMessageInfo {
            delay_shard_name: delay_shard_name.clone(),
            target_shard_name: target_topic.to_string(),
            offset,
            delay_timestamp: now_second() + delay_timestamp,
        };
        self.send_to_delay_queue(shard_no, &delay_info);

        // persist DelayInfo
        persist_delay_info(&self.message_storage_adapter, &self.namespace, delay_info).await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), CommonError> {
        for shard_no in 0..self.shard_num {
            if let Some(stop_send) = self.delay_queue_pop_thread.get(&shard_no) {
                stop_send.send(true)?;
            }
        }
        Ok(())
    }

    pub fn send_to_delay_queue(&self, shard_no: u64, delay_info: &DelayMessageInfo) {
        if let Some(mut delay_queue) = self.delay_queue_list.get_mut(&shard_no) {
            delay_queue.insert_at(
                delay_info.clone(),
                Instant::now() + Duration::from_secs(delay_info.delay_timestamp - now_second()),
            );
            record_mqtt_delay_queue_total_capacity_set(shard_no, delay_queue.capacity() as i64);
            record_mqtt_delay_queue_used_capacity_set(shard_no, delay_queue.len() as i64);
            record_mqtt_delay_queue_remaining_capacity_set(
                shard_no,
                (delay_queue.capacity() - delay_queue.len()) as i64,
            );
        }
    }

    pub fn get_shard_num(&self) -> u64 {
        self.shard_num
    }

    fn add_delay_queue_pop_thread(&self, shard_no: u64, stop_send: broadcast::Sender<bool>) {
        self.delay_queue_pop_thread.insert(shard_no, stop_send);
    }

    fn get_target_shard_no(&self) -> u64 {
        self.incr_no
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.shard_num
    }
}
