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

use crate::{delay::save_delay_index_info, recover::recover_delay_queue};
use crate::{
    delay::{init_inner_topic, save_delay_message},
    pop::spawn_delay_message_pop_threads,
};
use broker_core::cache::BrokerCacheManager;
use common_base::uuid::unique_id;
use common_base::{error::common::CommonError, tools::now_second};
use common_metrics::mqtt::statistics::{
    record_mqtt_delay_queue_remaining_capacity_set, record_mqtt_delay_queue_total_capacity_set,
    record_mqtt_delay_queue_used_capacity_set,
};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    delay_info::DelayMessageIndexInfo, storage::adapter_record::AdapterWriteRecord,
};
use std::{
    sync::{atomic::AtomicU32, Arc},
    time::Duration,
};
use storage_adapter::driver::StorageDriverManager;
use tokio::{sync::broadcast, time::Instant};
use tokio_util::time::DelayQueue;
use tracing::{error, info};

pub const DELAY_MESSAGE_FLAG: &str = "delay_message_flag";
pub const DELAY_MESSAGE_RECV_MS: &str = "delay_message_recv_ms";
pub const DELAY_MESSAGE_TARGET_MS: &str = "delay_message_target_ms";
pub const DELAY_MESSAGE_SAVE_MS: &str = "delay_message_save_ms";

pub async fn start_delay_message_manager_thread(
    delay_message_manager: &Arc<DelayMessageManager>,
    broker_cache: &Arc<BrokerCacheManager>,
) -> Result<(), CommonError> {
    delay_message_manager.start();

    init_inner_topic(delay_message_manager, broker_cache).await?;
    recover_delay_queue(delay_message_manager).await;
    spawn_delay_message_pop_threads(delay_message_manager, delay_message_manager.delay_queue_num);

    Ok(())
}

pub struct DelayMessageManager {
    pub client_pool: Arc<ClientPool>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub delay_queue_list: DashMap<u32, DelayQueue<DelayMessageIndexInfo>>,
    pub delay_queue_pop_thread: DashMap<u32, broadcast::Sender<bool>>,
    pub delay_queue_num: u32,
    pub incr_no: AtomicU32,
}

impl DelayMessageManager {
    pub async fn new(
        client_pool: Arc<ClientPool>,
        storage_driver_manager: Arc<StorageDriverManager>,
        delay_queue_num: u32,
    ) -> Result<Self, CommonError> {
        let driver = DelayMessageManager {
            client_pool,
            storage_driver_manager,
            delay_queue_list: DashMap::with_capacity(8),
            delay_queue_pop_thread: DashMap::with_capacity(8),
            incr_no: AtomicU32::new(0),
            delay_queue_num,
        };
        Ok(driver)
    }

    pub fn start(&self) {
        for shard_no in 0..self.delay_queue_num {
            self.delay_queue_list.insert(shard_no, DelayQueue::new());
        }
    }

    pub async fn send(
        &self,
        target_topic: &str,
        target_timestamp: u64,
        data: AdapterWriteRecord,
    ) -> Result<String, CommonError> {
        info!("0");
        let delay_message_id = unique_id();
        let offset =
            save_delay_message(&self.storage_driver_manager, &delay_message_id, data).await?;

        let delay_index_info = DelayMessageIndexInfo {
            unique_id: delay_message_id.clone(),
            target_topic_name: target_topic.to_string(),
            offset,
            target_timestamp,
        };

        save_delay_index_info(&self.storage_driver_manager, &delay_index_info).await?;

        self.send_to_delay_queue(&delay_index_info);

        Ok(delay_message_id)
    }

    pub async fn stop(&self) -> Result<(), CommonError> {
        for shard_no in 0..self.delay_queue_num {
            if let Some(stop_send) = self.delay_queue_pop_thread.get(&shard_no) {
                stop_send.send(true)?;
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    pub fn send_to_delay_queue(&self, delay_info: &DelayMessageIndexInfo) {
        let shard_no = self
            .incr_no
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.delay_queue_num;

        if let Some(mut delay_queue) = self.delay_queue_list.get_mut(&shard_no) {
            let current_time = now_second();
            let delay_duration = if delay_info.target_timestamp > current_time {
                Duration::from_secs(delay_info.target_timestamp - current_time)
            } else {
                Duration::from_secs(0)
            };
            let target_instant = Instant::now() + delay_duration;
            let expected_trigger_time = current_time + delay_duration.as_secs();

            info!(
                "Insert delay message to queue. unique_id={}, target_topic={}, shard_no={}, target_timestamp={}, current_time={}, delay_duration={}s, expected_trigger_time={}, time_match={}",
                delay_info.unique_id,
                delay_info.target_topic_name,
                shard_no,
                delay_info.target_timestamp,
                current_time,
                delay_duration.as_secs(),
                expected_trigger_time,
                expected_trigger_time == delay_info.target_timestamp
            );

            delay_queue.insert_at(delay_info.clone(), target_instant);

            let capacity = delay_queue.capacity() as i64;
            let used = delay_queue.len() as i64;
            drop(delay_queue);

            record_mqtt_delay_queue_total_capacity_set(shard_no, capacity);
            record_mqtt_delay_queue_used_capacity_set(shard_no, used);
            record_mqtt_delay_queue_remaining_capacity_set(shard_no, capacity - used);
        } else {
            error!(
                "Failed to send to delay queue: shard {} not found, message will be lost: \
                target={}, offset={}, delay_timestamp={}",
                shard_no,
                delay_info.target_topic_name,
                delay_info.offset,
                delay_info.target_timestamp
            );
        }
    }

    pub fn add_delay_queue_pop_thread(&self, shard_no: u32, stop_send: broadcast::Sender<bool>) {
        self.delay_queue_pop_thread.insert(shard_no, stop_send);
    }
}

#[cfg(test)]
mod test {}
