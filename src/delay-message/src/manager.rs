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

use crate::delay::{delete_delay_index_info, delete_delay_message, save_delay_index_info};
use crate::{
    delay::{init_inner_topic, save_delay_message},
    pop::spawn_delay_message_pop_threads,
    recover::recover_delay_queue,
};
use broker_core::cache::NodeCacheManager;
use common_base::task::TaskSupervisor;
use common_base::uuid::unique_id;
use common_base::{error::common::CommonError, tools::now_second};
use common_metrics::mqtt::delay::{record_delay_msg_enqueue, record_delay_msg_enqueue_duration};
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
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::Instant;
use tokio_util::time::delay_queue;
use tracing::debug;

/// Command sent from the manager to the per-shard pop thread.
pub(crate) enum ShardCmd {
    /// Insert a new message at the given instant. Pop thread replies with the queue Key.
    Insert(
        DelayMessageIndexInfo,
        Instant,
        oneshot::Sender<delay_queue::Key>,
    ),
    /// Remove a message by its queue key. Pop thread sends () when done.
    Delete(delay_queue::Key, oneshot::Sender<()>),
}

/// Sender half kept in the manager; pop thread owns the receiver.
pub(crate) type ShardCmdTx = mpsc::UnboundedSender<ShardCmd>;

pub const DELAY_MESSAGE_FLAG: &str = "delay_message_flag";
pub const DELAY_MESSAGE_RECV_MS: &str = "delay_message_recv_ms";
pub const DELAY_MESSAGE_TARGET_MS: &str = "delay_message_target_ms";
pub const DELAY_MESSAGE_SAVE_MS: &str = "delay_message_save_ms";

pub async fn start_delay_message_manager_thread(
    delay_message_manager: &Arc<DelayMessageManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    broker_cache: &Arc<NodeCacheManager>,
) -> Result<(), CommonError> {
    init_inner_topic(delay_message_manager, broker_cache).await?;

    spawn_delay_message_pop_threads(
        delay_message_manager,
        task_supervisor,
        delay_message_manager.delay_queue_num,
    );

    let recover_manager = delay_message_manager.clone();
    tokio::spawn(async move {
        recover_delay_queue(&recover_manager).await;
    });

    Ok(())
}

pub struct DelayMessageManager {
    pub client_pool: Arc<ClientPool>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    /// Per-shard command channel senders.
    pub(crate) shard_cmd_tx: DashMap<u32, ShardCmdTx>,
    pub delay_queue_pop_thread: DashMap<u32, broadcast::Sender<bool>>,
    pub delay_queue_num: u32,
    pub incr_no: AtomicU32,
    /// unique_id → (shard_no, queue key).
    message_key_map: DashMap<String, (u32, delay_queue::Key)>,
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
            shard_cmd_tx: DashMap::with_capacity(8),
            delay_queue_pop_thread: DashMap::with_capacity(8),
            incr_no: AtomicU32::new(0),
            delay_queue_num,
            message_key_map: DashMap::new(),
        };
        Ok(driver)
    }

    /// Called by pop.rs to register the command-channel sender for a shard.
    pub(crate) fn register_shard_cmd_tx(&self, shard_no: u32, tx: ShardCmdTx) {
        self.shard_cmd_tx.insert(shard_no, tx);
    }

    pub(crate) fn remove_message_key(&self, unique_id: &str) {
        self.message_key_map.remove(unique_id);
    }

    pub async fn send(
        &self,
        tenant: &str,
        target_topic: &str,
        target_timestamp: u64,
        data: AdapterWriteRecord,
    ) -> Result<String, CommonError> {
        let start = std::time::Instant::now();

        let delay_message_id = unique_id();
        let offset =
            save_delay_message(&self.storage_driver_manager, &delay_message_id, data).await?;

        let delay_index_info = DelayMessageIndexInfo {
            unique_id: delay_message_id.clone(),
            tenant: tenant.to_string(),
            target_topic_name: target_topic.to_string(),
            offset,
            target_timestamp,
        };

        save_delay_index_info(&self.storage_driver_manager, &delay_index_info).await?;

        self.send_to_delay_queue(&delay_index_info).await?;

        record_delay_msg_enqueue();
        record_delay_msg_enqueue_duration(start.elapsed().as_secs_f64() * 1000.0);

        Ok(delay_message_id)
    }

    /// Cancel a previously scheduled delay message by its unique_id.
    /// Removes the entry from the in-memory queue and deletes it from persistent storage.
    pub async fn cancel(&self, unique_id: &str) -> Result<(), CommonError> {
        let entry = match self.message_key_map.remove(unique_id) {
            Some(e) => e,
            None => {
                debug!(
                    "Delay message not found when cancelling, may have already been delivered: unique_id={}",
                    unique_id
                );
                return Ok(());
            }
        };
        let (_, (shard_no, key)) = entry;

        let tx = self
            .shard_cmd_tx
            .get(&shard_no)
            .map(|r| r.clone())
            .ok_or_else(|| {
                CommonError::CommonError(format!(
                    "Delay message shard {} cmd channel not found when cancelling unique_id={}",
                    shard_no, unique_id
                ))
            })?;

        let (done_tx, done_rx) = oneshot::channel();
        tx.send(ShardCmd::Delete(key, done_tx))
            .map_err(|e| CommonError::CommonError(format!("shard cmd send error: {}", e)))?;

        // Wait until pop thread has actually removed the entry from the DelayQueue.
        let _ = done_rx.await;

        // Clean up persistent storage.
        let delay_info = DelayMessageIndexInfo {
            unique_id: unique_id.to_string(),
            tenant: String::new(),
            target_topic_name: String::new(),
            offset: 0,
            target_timestamp: 0,
        };
        delete_delay_index_info(&self.storage_driver_manager, &delay_info).await?;
        delete_delay_message(&self.storage_driver_manager, unique_id).await?;

        debug!("Delay message cancelled: unique_id={}", unique_id);
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

    /// Send delay message to appropriate queue shard via channel — non-blocking, no Mutex.
    /// Awaits the queue Key reply so that a subsequent cancel() will never miss the entry.
    pub async fn send_to_delay_queue(
        &self,
        delay_info: &DelayMessageIndexInfo,
    ) -> Result<(), CommonError> {
        let shard_no = self
            .incr_no
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.delay_queue_num;

        let tx = self
            .shard_cmd_tx
            .get(&shard_no)
            .map(|r| r.clone())
            .ok_or_else(|| {
                CommonError::CommonError(format!(
                    "Delay message shard {} cmd channel not found: unique_id={}",
                    shard_no, delay_info.unique_id
                ))
            })?;

        let current_time = now_second();
        let delay_duration = if delay_info.target_timestamp > current_time {
            Duration::from_secs(delay_info.target_timestamp - current_time)
        } else {
            Duration::from_secs(0)
        };
        let target_instant = Instant::now() + delay_duration;

        let (key_tx, key_rx) = oneshot::channel();
        tx.send(ShardCmd::Insert(delay_info.clone(), target_instant, key_tx))
            .map_err(|e| {
                CommonError::CommonError(format!(
                    "Failed to send Insert cmd to delay message shard {}: unique_id={}, error={}",
                    shard_no, delay_info.unique_id, e
                ))
            })?;

        match key_rx.await {
            Ok(key) => {
                self.message_key_map
                    .insert(delay_info.unique_id.clone(), (shard_no, key));
            }
            Err(_) => {
                return Err(CommonError::CommonError(format!(
                    "Pop thread dropped before replying with key: unique_id={}",
                    delay_info.unique_id
                )));
            }
        }

        Ok(())
    }

    pub fn add_delay_queue_pop_thread(&self, shard_no: u32, stop_send: broadcast::Sender<bool>) {
        self.delay_queue_pop_thread.insert(shard_no, stop_send);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use common_base::tools::now_second;

    fn create_test_delay_info(unique_id: &str, target_timestamp: u64) -> DelayMessageIndexInfo {
        DelayMessageIndexInfo {
            unique_id: unique_id.to_string(),
            tenant: "test_tenant".to_string(),
            target_topic_name: "test_topic".to_string(),
            offset: 12345,
            target_timestamp,
        }
    }

    #[tokio::test]
    async fn test_delay_queue_basic_operations() {
        use tokio_util::time::DelayQueue;
        let mut delay_queue: DelayQueue<DelayMessageIndexInfo> = DelayQueue::new();

        let delay_info = create_test_delay_info("test_1", now_second() + 3600);
        delay_queue.insert_at(
            delay_info.clone(),
            Instant::now() + Duration::from_secs(3600),
        );

        assert_eq!(delay_queue.len(), 1);
    }

    #[tokio::test]
    async fn test_delay_queue_expired_message() {
        use tokio_util::time::DelayQueue;
        let mut delay_queue: DelayQueue<DelayMessageIndexInfo> = DelayQueue::new();

        let delay_info = create_test_delay_info("expired_test", now_second() - 10);
        delay_queue.insert_at(delay_info.clone(), Instant::now());

        assert_eq!(delay_queue.len(), 1);
    }

    #[tokio::test]
    async fn test_delay_queue_multiple_messages() {
        use tokio_util::time::DelayQueue;
        let mut delay_queue: DelayQueue<DelayMessageIndexInfo> = DelayQueue::new();

        for i in 0..5 {
            let delay_info = create_test_delay_info(&format!("test_{}", i), now_second() + 3600);
            delay_queue.insert_at(delay_info, Instant::now() + Duration::from_secs(3600));
        }

        assert_eq!(delay_queue.len(), 5);
    }
}
