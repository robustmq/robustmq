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

use crate::delay::{delete_delay_task_index, save_delay_task_index};
use crate::DelayTask;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_metrics::mqtt::delay_task::record_delay_task_created;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use std::sync::{atomic::AtomicU32, Arc};
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::{broadcast, Mutex, Semaphore};
use tokio::time::Instant;
use tokio_util::time::delay_queue;
use tokio_util::time::DelayQueue;
use tracing::{debug, error, warn};

pub type SharedDelayQueue = Arc<Mutex<DelayQueue<DelayTask>>>;

#[derive(Clone)]
pub struct DelayTaskManager {
    pub client_pool: Arc<ClientPool>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub delay_queue_list: DashMap<u32, SharedDelayQueue>,
    pub delay_queue_pop_thread: DashMap<u32, broadcast::Sender<bool>>,
    pub delay_queue_num: u32,
    pub handler_semaphore: Arc<Semaphore>,
    incr_no: Arc<AtomicU32>,
    task_key_map: DashMap<String, (u32, delay_queue::Key)>,
}

impl DelayTaskManager {
    pub fn new(
        client_pool: Arc<ClientPool>,
        storage_driver_manager: Arc<StorageDriverManager>,
        delay_queue_num: u32,
        max_handler_concurrency: usize,
    ) -> Self {
        DelayTaskManager {
            client_pool,
            storage_driver_manager,
            delay_queue_list: DashMap::with_capacity(8),
            delay_queue_pop_thread: DashMap::with_capacity(8),
            incr_no: Arc::new(AtomicU32::new(0)),
            delay_queue_num,
            handler_semaphore: Arc::new(Semaphore::new(max_handler_concurrency)),
            task_key_map: DashMap::new(),
        }
    }

    pub fn start(&self) {
        for shard_no in 0..self.delay_queue_num {
            self.delay_queue_list
                .insert(shard_no, Arc::new(Mutex::new(DelayQueue::new())));
        }
    }

    pub async fn create_task(&self, task: DelayTask) -> Result<String, CommonError> {
        if self.task_key_map.contains_key(&task.task_id) {
            self.delete_task(&task.task_id).await?;
            debug!(
                "Replaced existing delay task: task_id={}, task_type={}",
                task.task_id,
                task.task_type_name()
            );
        }

        if task.persistent {
            save_delay_task_index(&self.storage_driver_manager, &task).await?;
        }
        self.enqueue_task(&task).await;
        record_delay_task_created();
        Ok(task.task_id.clone())
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), CommonError> {
        let (_, (shard_no, key)) = match self.task_key_map.remove(task_id) {
            Some(entry) => entry,
            None => {
                warn!(
                    "Delay task not found when deleting, may have already been executed: task_id={}",
                    task_id
                );
                return Ok(());
            }
        };

        let queue_arc = self.delay_queue_list.get(&shard_no).ok_or_else(|| {
            CommonError::CommonError(format!(
                "Delay task queue shard {} not found when deleting task_id={}",
                shard_no, task_id
            ))
        })?;

        let mut delay_queue = queue_arc.lock().await;
        let task = delay_queue.remove(&key).into_inner();
        drop(delay_queue);

        if task.persistent {
            delete_delay_task_index(&self.storage_driver_manager, task_id).await?;
        }

        debug!(
            "Delay task deleted: task_id={}, task_type={}",
            task_id,
            task.task_type_name()
        );
        Ok(())
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

    pub(crate) async fn enqueue_task(&self, task: &DelayTask) {
        let shard_no = self
            .incr_no
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.delay_queue_num;

        let queue_arc = if let Some(q) = self.delay_queue_list.get(&shard_no) {
            q.clone()
        } else {
            error!(
                "Failed to enqueue delay task: shard {} not found, task will be lost: \
                task_id={}, task_type={}",
                shard_no,
                task.task_id,
                task.task_type_name()
            );
            return;
        };

        let current_time = now_second();
        let delay_duration = if task.delay_target_time > current_time {
            Duration::from_secs(task.delay_target_time - current_time)
        } else {
            Duration::from_secs(0)
        };
        let target_instant = Instant::now() + delay_duration;

        debug!(
            "Insert delay task to queue. task_id={}, task_type={}, shard_no={}, \
            delay_target_time={}, current_time={}, delay_duration={}s",
            task.task_id,
            task.task_type_name(),
            shard_no,
            task.delay_target_time,
            current_time,
            delay_duration.as_secs(),
        );

        let mut delay_queue = queue_arc.lock().await;
        let key = delay_queue.insert_at(task.clone(), target_instant);
        drop(delay_queue);

        self.task_key_map
            .insert(task.task_id.clone(), (shard_no, key));
    }

    pub fn contains_task(&self, task_id: &str) -> bool {
        self.task_key_map.contains_key(task_id)
    }

    pub(crate) fn remove_task_key(&self, task_id: &str) {
        self.task_key_map.remove(task_id);
    }

    pub fn add_delay_queue_pop_thread(&self, shard_no: u32, stop_send: broadcast::Sender<bool>) {
        self.delay_queue_pop_thread.insert(shard_no, stop_send);
    }
}
