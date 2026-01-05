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

use crate::delay::{
    get_delay_message_shard_name, init_delay_message_shard, save_delay_message,
    start_delay_message_pop,
};
use common_base::{
    error::common::CommonError,
    tools::{now_second, unique_id},
};
use common_config::storage::StorageAdapterType;
use common_metrics::mqtt::statistics::{
    record_mqtt_delay_queue_remaining_capacity_set, record_mqtt_delay_queue_total_capacity_set,
    record_mqtt_delay_queue_used_capacity_set,
};
use dashmap::DashMap;
use metadata_struct::{
    delay_info::DelayMessageIndexInfo, storage::adapter_record::AdapterWriteRecord,
};
use std::{
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use storage_adapter::driver::{ArcStorageAdapter, StorageDriverManager};
use tokio::{sync::broadcast, time::Instant};
use tokio_util::time::DelayQueue;
use tracing::debug;

use crate::{
    delay::save_delay_index_info, driver::get_delay_message_storage_driver,
    recover::start_recover_delay_queue,
};

pub async fn start_delay_message_manager(
    delay_message_manager: &Arc<DelayMessageManager>,
    shard_num: u64,
) -> Result<(), CommonError> {
    delay_message_manager.start().await;

    init_delay_message_shard(
        &delay_message_manager.message_storage_adapter,
        &delay_message_manager.storage_adapter_type,
        shard_num,
    )
    .await?;

    start_recover_delay_queue(
        delay_message_manager,
        &delay_message_manager.message_storage_adapter,
        shard_num,
    );

    start_delay_message_pop(
        delay_message_manager,
        &delay_message_manager.message_storage_adapter,
        shard_num,
    );

    Ok(())
}

pub struct DelayMessageManager {
    pub message_storage_adapter: ArcStorageAdapter,
    pub storage_adapter_type: StorageAdapterType,
    shard_num: u64,
    incr_no: AtomicU64,
    pub delay_queue_list: DashMap<u64, DelayQueue<DelayMessageIndexInfo>>,
    delay_queue_pop_thread: DashMap<u64, broadcast::Sender<bool>>,
    shard_engine_type_list: DashMap<String, StorageAdapterType>,
}

impl DelayMessageManager {
    pub async fn new(
        storage_driver_manager: Arc<StorageDriverManager>,
        storage_adapter_type: StorageAdapterType,
        shard_num: u64,
    ) -> Result<Self, CommonError> {
        let message_storage_adapter =
            get_delay_message_storage_driver(&storage_driver_manager, &storage_adapter_type)?;
        let driver = DelayMessageManager {
            shard_num,
            message_storage_adapter,
            storage_adapter_type,
            incr_no: AtomicU64::new(0),
            delay_queue_list: DashMap::with_capacity(shard_num as usize),
            delay_queue_pop_thread: DashMap::with_capacity(shard_num as usize),
            shard_engine_type_list: DashMap::with_capacity(8),
        };
        Ok(driver)
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
        data: AdapterWriteRecord,
    ) -> Result<(), CommonError> {
        let shard_no = self.get_target_shard_no();
        let delay_shard_name = get_delay_message_shard_name(shard_no);

        // Persist DelayMessage
        let offset =
            save_delay_message(&self.message_storage_adapter, &delay_shard_name, data).await?;

        let delay_index_info = DelayMessageIndexInfo {
            unique_id: unique_id(),
            delay_shard_name: delay_shard_name.clone(),
            target_shard_name: target_topic.to_string(),
            offset,
            delay_timestamp: now_second() + delay_timestamp,
            shard_no,
        };

        // persist DelayIndexInfo
        save_delay_index_info(&self.message_storage_adapter, &delay_index_info).await?;

        // DelayInfo into delay queue
        self.send_to_delay_queue(shard_no, &delay_index_info);

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

    pub fn send_to_delay_queue(&self, shard_no: u64, delay_info: &DelayMessageIndexInfo) {
        if let Some(mut delay_queue) = self.delay_queue_list.get_mut(&shard_no) {
            let now = now_second();
            let delay_duration = if delay_info.delay_timestamp > now {
                Duration::from_secs(delay_info.delay_timestamp - now)
            } else {
                Duration::from_secs(0)
            };

            debug!(
                "Adding message to delay queue: shard_no={}, target={}, delay={}s, will_expire_at={}",
                shard_no,
                delay_info.target_shard_name,
                delay_duration.as_secs(),
                delay_info.delay_timestamp
            );

            delay_queue.insert_at(delay_info.clone(), Instant::now() + delay_duration);
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

    pub fn add_shard_engine_type_list(
        &self,
        shard_name: String,
        engine_storage_type: StorageAdapterType,
    ) {
        self.shard_engine_type_list
            .insert(shard_name, engine_storage_type);
    }

    pub fn remove_shard_engine_type_list(&self, shard_name: &str) {
        self.shard_engine_type_list.remove(shard_name);
    }

    pub fn get_shard_engine_type_list(&self, shard_name: &str) -> Option<StorageAdapterType> {
        if let Some(engine_type) = self.shard_engine_type_list.get(shard_name) {
            return Some(*engine_type);
        }
        None
    }

    pub fn add_delay_queue_pop_thread(&self, shard_no: u64, stop_send: broadcast::Sender<bool>) {
        self.delay_queue_pop_thread.insert(shard_no, stop_send);
    }

    fn get_target_shard_no(&self) -> u64 {
        self.incr_no
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
            % self.shard_num
    }
}
