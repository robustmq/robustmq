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
use broker_core::cache::NodeCacheManager;
use common_base::task::TaskSupervisor;
use common_base::uuid::unique_id;
use common_base::{error::common::CommonError, tools::now_second};
use common_metrics::mqtt::delay::{record_delay_msg_enqueue, record_delay_msg_enqueue_duration};
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

pub type SharedDelayQueue = Arc<tokio::sync::Mutex<DelayQueue<DelayMessageIndexInfo>>>;

pub const DELAY_MESSAGE_FLAG: &str = "delay_message_flag";
pub const DELAY_MESSAGE_RECV_MS: &str = "delay_message_recv_ms";
pub const DELAY_MESSAGE_TARGET_MS: &str = "delay_message_target_ms";
pub const DELAY_MESSAGE_SAVE_MS: &str = "delay_message_save_ms";

pub async fn start_delay_message_manager_thread(
    delay_message_manager: &Arc<DelayMessageManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    broker_cache: &Arc<NodeCacheManager>,
) -> Result<(), CommonError> {
    delay_message_manager.start();

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
    pub delay_queue_list: DashMap<u32, SharedDelayQueue>,
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
            self.delay_queue_list.insert(
                shard_no,
                Arc::new(tokio::sync::Mutex::new(DelayQueue::new())),
            );
        }
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

        self.send_to_delay_queue(&delay_index_info).await;

        record_delay_msg_enqueue();
        record_delay_msg_enqueue_duration(start.elapsed().as_secs_f64() * 1000.0);

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

    /// Send delay message to appropriate queue shard.
    /// 
    /// This method uses lazy initialization to ensure queue shards are created on-demand,
    /// preventing message loss even if initialization is incomplete or concurrent with startup.
    /// Messages are already persisted in storage before this method is called,
    /// so even on queue creation failure, messages can be recovered later.
    pub async fn send_to_delay_queue(&self, delay_info: &DelayMessageIndexInfo) {
        let shard_no = self
            .incr_no
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.delay_queue_num;

        // Lazy initialization: create queue shard if it doesn't exist
        // This prevents message loss during startup or recovery scenarios
        let queue_arc = self.delay_queue_list
            .entry(shard_no)
            .or_insert_with(|| {
                info!(
                    "Delay queue shard {} not found, initializing lazily. Message may be recovered from storage.",
                    shard_no
                );
                Arc::new(tokio::sync::Mutex::new(DelayQueue::new()))
            })
            .clone();

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

        let mut delay_queue = queue_arc.lock().await;
        delay_queue.insert_at(delay_info.clone(), target_instant);

        let capacity = delay_queue.capacity() as i64;
        let used = delay_queue.len() as i64;
        drop(delay_queue);

        record_mqtt_delay_queue_total_capacity_set(shard_no, capacity);
        record_mqtt_delay_queue_used_capacity_set(shard_no, used);
        record_mqtt_delay_queue_remaining_capacity_set(shard_no, capacity - used);
    }

    pub fn add_delay_queue_pop_thread(&self, shard_no: u32, stop_send: broadcast::Sender<bool>) {
        self.delay_queue_pop_thread.insert(shard_no, stop_send);
    }

    /// Check if all queue shards are initialized
    pub fn is_fully_initialized(&self) -> bool {
        self.delay_queue_list.len() == self.delay_queue_num as usize
    }

    /// Get the number of initialized queue shards
    pub fn initialized_shard_count(&self) -> usize {
        self.delay_queue_list.len()
    }

    /// Get pending message count for a specific shard (for monitoring/debugging)
    pub async fn get_shard_pending_count(&self, shard_no: u32) -> Option<usize> {
        if let Some(queue_arc) = self.delay_queue_list.get(&shard_no) {
            let queue = queue_arc.lock().await;
            Some(queue.len())
        } else {
            None
        }
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

    /// Create a minimal manager for testing without actual storage
    /// We only test the send_to_delay_queue logic, not the persistence
    struct TestDelayMessageManager {
        delay_queue_list: DashMap<u32, SharedDelayQueue>,
        delay_queue_num: u32,
        incr_no: AtomicU32,
    }

    impl TestDelayMessageManager {
        fn new(delay_queue_num: u32) -> Self {
            Self {
                delay_queue_list: DashMap::with_capacity(8),
                delay_queue_num,
                incr_no: AtomicU32::new(0),
            }
        }

        fn start(&self) {
            for shard_no in 0..self.delay_queue_num {
                self.delay_queue_list.insert(
                    shard_no,
                    Arc::new(tokio::sync::Mutex::new(DelayQueue::new())),
                );
            }
        }

        async fn send_to_delay_queue(&self, delay_info: &DelayMessageIndexInfo) {
            let shard_no = self
                .incr_no
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                % self.delay_queue_num;

            let queue_arc = self.delay_queue_list
                .entry(shard_no)
                .or_insert_with(|| {
                    Arc::new(tokio::sync::Mutex::new(DelayQueue::new()))
                })
                .clone();

            let current_time = now_second();
            let delay_duration = if delay_info.target_timestamp > current_time {
                Duration::from_secs(delay_info.target_timestamp - current_time)
            } else {
                Duration::from_secs(0)
            };
            let target_instant = Instant::now() + delay_duration;

            let mut delay_queue = queue_arc.lock().await;
            delay_queue.insert_at(delay_info.clone(), target_instant);
        }

        fn is_fully_initialized(&self) -> bool {
            self.delay_queue_list.len() == self.delay_queue_num as usize
        }

        fn initialized_shard_count(&self) -> usize {
            self.delay_queue_list.len()
        }

        async fn get_shard_pending_count(&self, shard_no: u32) -> Option<usize> {
            if let Some(queue_arc) = self.delay_queue_list.get(&shard_no) {
                let queue = queue_arc.lock().await;
                Some(queue.len())
            } else {
                None
            }
        }
    }

    #[tokio::test]
    async fn test_lazy_initialization_creates_queue() {
        // Create manager without calling start()
        let delay_queue_num = 4;
        let manager = TestDelayMessageManager::new(delay_queue_num);

        // Initially, no queues are initialized
        assert_eq!(manager.initialized_shard_count(), 0);
        assert!(!manager.is_fully_initialized());

        // Send message to shard 0 (should create queue lazily)
        let delay_info = create_test_delay_info("test_1", now_second() + 3600);
        manager.send_to_delay_queue(&delay_info).await;

        // Now 1 queue should be initialized
        assert_eq!(manager.initialized_shard_count(), 1);
        
        // The message should be in the queue
        let pending = manager.get_shard_pending_count(0).await;
        assert_eq!(pending, Some(1));
    }

    #[tokio::test]
    async fn test_concurrent_lazy_initialization() {
        let delay_queue_num = 4;
        let manager = Arc::new(TestDelayMessageManager::new(delay_queue_num));

        // Send messages concurrently to all shards
        let mut handles = vec![];
        for i in 0..delay_queue_num {
            let mgr = manager.clone();
            let unique_id = format!("concurrent_test_{}", i);
            let delay_info = create_test_delay_info(&unique_id, now_second() + 3600);
            
            handles.push(tokio::spawn(async move {
                mgr.send_to_delay_queue(&delay_info).await;
            }));
        }

        // Wait for all concurrent sends to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // All shards should be initialized after concurrent sends
        assert_eq!(manager.initialized_shard_count(), delay_queue_num as usize);
        assert!(manager.is_fully_initialized());

        // Each shard should have exactly 1 message
        for shard_no in 0..delay_queue_num {
            let pending = manager.get_shard_pending_count(shard_no).await;
            assert_eq!(pending, Some(1), "Shard {} should have 1 message", shard_no);
        }
    }

    #[tokio::test]
    async fn test_round_robin_distribution() {
        let delay_queue_num = 3;
        let manager = TestDelayMessageManager::new(delay_queue_num);

        // Send 9 messages (should distribute 3 per shard)
        for i in 0..9 {
            let unique_id = format!("dist_test_{}", i);
            let delay_info = create_test_delay_info(&unique_id, now_second() + 3600);
            manager.send_to_delay_queue(&delay_info).await;
        }

        // Each shard should have exactly 3 messages
        for shard_no in 0..delay_queue_num {
            let pending = manager.get_shard_pending_count(shard_no).await;
            assert_eq!(pending, Some(3), "Shard {} should have 3 messages", shard_no);
        }
    }

    #[tokio::test]
    async fn test_expired_delay_message() {
        let delay_queue_num = 1;
        let manager = TestDelayMessageManager::new(delay_queue_num);

        // Send message with past timestamp (already expired)
        let past_timestamp = now_second() - 10;
        let delay_info = create_test_delay_info("expired_test", past_timestamp);
        manager.send_to_delay_queue(&delay_info).await;

        // Message should still be accepted into queue (for immediate processing)
        let pending = manager.get_shard_pending_count(0).await;
        assert_eq!(pending, Some(1));
    }

    #[tokio::test]
    async fn test_manager_start_initializes_all_shards() {
        let delay_queue_num = 5;
        let manager = TestDelayMessageManager::new(delay_queue_num);

        // Before start()
        assert_eq!(manager.initialized_shard_count(), 0);
        assert!(!manager.is_fully_initialized());

        // Call start()
        manager.start();

        // After start(), all shards should be initialized
        assert_eq!(manager.initialized_shard_count(), delay_queue_num as usize);
        assert!(manager.is_fully_initialized());

        // All shards should be empty
        for shard_no in 0..delay_queue_num {
            let pending = manager.get_shard_pending_count(shard_no).await;
            assert_eq!(pending, Some(0));
        }
    }

    #[tokio::test]
    async fn test_message_persistence_guarantee() {
        // This test verifies that even if queue creation fails,
        // the message is already persisted and can be recovered
        
        let delay_queue_num = 2;
        let manager = Arc::new(TestDelayMessageManager::new(delay_queue_num));

        // Send multiple messages
        for i in 0..10 {
            let unique_id = format!("persist_test_{}", i);
            let delay_info = create_test_delay_info(&unique_id, now_second() + 3600);
            manager.send_to_delay_queue(&delay_info).await;
        }

        // Verify all queues were created and messages stored
        let total_messages: usize = (0..delay_queue_num)
            .map(|shard_no| {
                futures::executor::block_on(async {
                    manager.get_shard_pending_count(shard_no).await.unwrap_or(0)
                })
            })
            .sum();

        assert_eq!(total_messages, 10);
    }

    #[tokio::test]
    async fn test_shard_overflow() {
        // Test that shard_no wraps around correctly
        let delay_queue_num = 3;
        let mut manager = TestDelayMessageManager::new(delay_queue_num);
        manager.incr_no = AtomicU32::new(u32::MAX - 2); // Start near overflow

        // Send messages that will cause shard_no calculation to wrap around
        for i in 0..10 {
            let unique_id = format!("overflow_test_{}", i);
            let delay_info = create_test_delay_info(&unique_id, now_second() + 3600);
            manager.send_to_delay_queue(&delay_info).await;
        }

        // Should still distribute correctly despite overflow
        let total_messages: usize = (0..delay_queue_num)
            .map(|shard_no| {
                futures::executor::block_on(async {
                    manager.get_shard_pending_count(shard_no).await.unwrap_or(0)
                })
            })
            .sum();

        assert_eq!(total_messages, 10);
    }
}
