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
use tokio::sync::{broadcast, mpsc, oneshot, Semaphore};
use tokio::time::Instant;
use tokio_util::time::delay_queue;
use tracing::{debug, error, warn};

/// Command sent from enqueue_task / delete_task to the per-shard pop thread.
pub(crate) enum ShardCmd {
    /// Insert a new task. Pop thread replies with the queue Key via the oneshot sender.
    Insert(DelayTask, Instant, oneshot::Sender<delay_queue::Key>),
    /// Remove a task by its queue key. Pop thread sends () when done.
    Delete(delay_queue::Key, oneshot::Sender<()>),
}

/// Sender half kept in the manager; pop thread owns the receiver.
pub(crate) type ShardCmdTx = mpsc::UnboundedSender<ShardCmd>;

#[derive(Clone)]
pub struct DelayTaskManager {
    pub client_pool: Arc<ClientPool>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    /// Per-shard command channel senders (Insert / Delete).
    pub(crate) shard_cmd_tx: DashMap<u32, ShardCmdTx>,
    pub delay_queue_pop_thread: DashMap<u32, broadcast::Sender<bool>>,
    pub delay_queue_num: u32,
    pub handler_semaphore: Arc<Semaphore>,
    incr_no: Arc<AtomicU32>,
    /// task_id → (shard_no, queue key, persistent).
    task_key_map: DashMap<String, (u32, delay_queue::Key, bool)>,
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
            shard_cmd_tx: DashMap::with_capacity(8),
            delay_queue_pop_thread: DashMap::with_capacity(8),
            incr_no: Arc::new(AtomicU32::new(0)),
            delay_queue_num,
            handler_semaphore: Arc::new(Semaphore::new(max_handler_concurrency)),
            task_key_map: DashMap::new(),
        }
    }

    /// Called by pop.rs to register the command-channel sender for a shard.
    pub(crate) fn register_shard_cmd_tx(&self, shard_no: u32, tx: ShardCmdTx) {
        self.shard_cmd_tx.insert(shard_no, tx);
    }

    pub(crate) fn remove_task_key(&self, task_id: &str) {
        self.task_key_map.remove(task_id);
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
        let entry = match self.task_key_map.remove(task_id) {
            Some(e) => e,
            None => {
                warn!(
                    "Delay task not found when deleting, may have already been executed: task_id={}",
                    task_id
                );
                return Ok(());
            }
        };
        let (_, (shard_no, key, persistent)) = entry;

        let tx = self
            .shard_cmd_tx
            .get(&shard_no)
            .map(|r| r.clone())
            .ok_or_else(|| {
                CommonError::CommonError(format!(
                    "Delay task shard {} cmd channel not found when deleting task_id={}",
                    shard_no, task_id
                ))
            })?;

        let (done_tx, done_rx) = oneshot::channel();
        tx.send(ShardCmd::Delete(key, done_tx))
            .map_err(|e| CommonError::CommonError(format!("shard cmd send error: {}", e)))?;

        // Wait until pop thread has actually removed the entry from the DelayQueue.
        // This prevents a race where delete completes but the task still fires.
        let _ = done_rx.await;

        if persistent {
            delete_delay_task_index(&self.storage_driver_manager, task_id).await?;
        }

        debug!("Delay task deleted: task_id={}", task_id);
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

    /// Sends Insert command to the shard's channel and awaits the queue Key reply.
    /// Returns only after the pop thread has inserted the task and the key is
    /// recorded in task_key_map, so a subsequent delete_task will never miss it.
    pub(crate) async fn enqueue_task(&self, task: &DelayTask) {
        let shard_no = self
            .incr_no
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.delay_queue_num;

        let tx = if let Some(t) = self.shard_cmd_tx.get(&shard_no) {
            t.clone()
        } else {
            error!(
                "Failed to enqueue delay task: shard {} cmd channel not found, task will be lost: \
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
            "Enqueue delay task. task_id={}, task_type={}, shard_no={}, \
            delay_target_time={}, current_time={}, delay_duration={}s",
            task.task_id,
            task.task_type_name(),
            shard_no,
            task.delay_target_time,
            current_time,
            delay_duration.as_secs(),
        );

        let (key_tx, key_rx) = oneshot::channel();
        if let Err(e) = tx.send(ShardCmd::Insert(task.clone(), target_instant, key_tx)) {
            error!(
                "Failed to send Insert cmd to shard {}: task_id={}, error={}",
                shard_no, task.task_id, e
            );
            return;
        }

        match key_rx.await {
            Ok(key) => {
                self.task_key_map
                    .insert(task.task_id.clone(), (shard_no, key, task.persistent));
            }
            Err(_) => {
                error!(
                    "Pop thread dropped before replying with key: task_id={}",
                    task.task_id
                );
            }
        }
    }

    pub fn contains_task(&self, task_id: &str) -> bool {
        self.task_key_map.contains_key(task_id)
    }

    pub fn add_delay_queue_pop_thread(&self, shard_no: u32, stop_send: broadcast::Sender<bool>) {
        self.delay_queue_pop_thread.insert(shard_no, stop_send);
    }
}
