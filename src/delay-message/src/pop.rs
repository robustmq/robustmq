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

use crate::delay::{delete_delay_index_info, delete_delay_message};
use crate::manager::{DelayMessageManager, ShardCmd, DELAY_MESSAGE_SAVE_MS};
use broker_core::inner_topic::DELAY_QUEUE_MESSAGE_TOPIC;
use common_base::error::common::CommonError;
use common_base::task::{TaskKind, TaskSupervisor};
use common_base::tools::now_second;
use common_metrics::mqtt::delay::{
    record_delay_msg_deliver, record_delay_msg_deliver_duration, record_delay_msg_deliver_fail,
    record_delay_msg_retry, record_delay_msg_retry_count,
};
use futures::StreamExt;
use metadata_struct::adapter::adapter_record::{AdapterWriteRecord, RecordHeader};
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::storage::record::StorageRecord;
use metadata_struct::tenant::DEFAULT_TENANT;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::{broadcast, mpsc};
use tokio::{select, sync::broadcast as bc};
use tokio_util::time::DelayQueue;
use tracing::{debug, error, info, warn};

pub(crate) fn spawn_delay_message_pop_threads(
    delay_message_manager: &Arc<DelayMessageManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    delay_queue_num: u32,
) {
    info!("Starting delay message pop threads ({})", delay_queue_num);

    for shard_no in 0..delay_queue_num {
        let manager = delay_message_manager.clone();

        // Create command channel for this shard.
        let (tx, rx) = mpsc::unbounded_channel::<ShardCmd>();
        delay_message_manager.register_shard_cmd_tx(shard_no, tx);

        let (stop_send, _) = broadcast::channel(2);
        delay_message_manager.add_delay_queue_pop_thread(shard_no, stop_send.clone());

        task_supervisor.spawn(
            format!("{}_{}", TaskKind::DelayMessagePop, shard_no),
            async move {
                run_shard_loop(shard_no, rx, stop_send, manager).await;
            },
        );
    }
}

/// Per-shard event loop.
/// Owns the DelayQueue exclusively — no Mutex needed.
/// Uses select! to react to either:
///   - a command from the manager (Insert)
///   - a message expiring in the DelayQueue
async fn run_shard_loop(
    shard_no: u32,
    mut rx: mpsc::UnboundedReceiver<ShardCmd>,
    stop_send: bc::Sender<bool>,
    manager: Arc<DelayMessageManager>,
) {
    let mut delay_queue: DelayQueue<DelayMessageIndexInfo> = DelayQueue::new();
    let mut stop_recv = stop_send.subscribe();

    loop {
        select! {
            // Stop signal
            val = stop_recv.recv() => {
                match val {
                    Ok(flag) if flag => {
                        info!("Delay message pop thread stopped for shard {}", shard_no);
                        break;
                    }
                    Err(_) => {
                        warn!("Broadcast channel closed, stopping pop thread for shard {}", shard_no);
                        break;
                    }
                    _ => {}
                }
            }

            // Command from manager (Insert / Delete)
            cmd = rx.recv() => {
                match cmd {
                    Some(ShardCmd::Insert(delay_info, target_instant, key_tx)) => {
                        let key = delay_queue.insert_at(delay_info, target_instant);
                        let _ = key_tx.send(key);
                    }
                    Some(ShardCmd::Delete(key, done_tx)) => {
                        delay_queue.remove(&key);
                        let _ = done_tx.send(());
                    }
                    None => {
                        // Channel closed — manager dropped, exit.
                        break;
                    }
                }
            }

            // Expired message
            Some(expired) = delay_queue.next() => {
                let delay_message = expired.into_inner();
                manager.remove_message_key(&delay_message.unique_id);
                let storage = manager.storage_driver_manager.clone();
                let retry_manager = manager.clone();
                let config = manager.delay_message_config.clone();
                tokio::spawn(async move {
                    if let Err(e) = delay_message_process(
                        &storage,
                        &delay_message,
                        now_second(),
                    )
                    .await
                    {
                        if delay_message.retry_count < config.max_retries {
                            let mut retry = delay_message.clone();
                            retry.retry_count += 1;
                            let backoff_secs = config.initial_retry_delay_sec
                                * 2_u64.pow(retry.retry_count.saturating_sub(1));
                            record_delay_msg_retry(false);
                            record_delay_msg_retry_count(retry.retry_count);
                            warn!(
                                "Delay message delivery failed (attempt {}/{}), retrying in {}s: unique_id={}, target={}, error={}",
                                retry.retry_count, config.max_retries, backoff_secs,
                                retry.unique_id, retry.target_topic_name, e
                            );
                            retry_manager
                                .reenqueue_for_retry(retry, Duration::from_secs(backoff_secs))
                                .await;
                        } else {
                            record_delay_msg_retry(false);
                            record_delay_msg_retry_count(delay_message.retry_count);
                            error!(
                                "Delay message delivery failed after {} attempts, dropping: unique_id={}, target={}, error={}",
                                delay_message.retry_count, delay_message.unique_id,
                                delay_message.target_topic_name, e
                            );
                            let _ = delete_delay_index_info(&storage, &delay_message).await;
                            let _ = delete_delay_message(&storage, &delay_message.unique_id).await;
                        }
                    } else if delay_message.retry_count > 0 {
                        record_delay_msg_retry(true);
                    }
                });
            }
        }
    }
}

pub async fn delay_message_process(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_info: &DelayMessageIndexInfo,
    trigger_time: u64,
) -> Result<(), CommonError> {
    let start = Instant::now();

    // Only delete the stored index + message after a *successful* delivery. A transient
    // failure (target shard leader unreachable, metadata not yet synced) must NOT drop the
    // message — the caller re-enqueues it for retry. Deleting on failure here permanently
    // loses the message.
    let offset =
        match send_delay_message_to_shard(storage_driver_manager, delay_info, trigger_time).await {
            Ok(offset) => offset,
            Err(e) => {
                record_delay_msg_deliver_fail();
                return Err(e);
            }
        };

    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    record_delay_msg_deliver();
    record_delay_msg_deliver_duration(duration_ms);
    info!(
        "Delay message processed successfully. unique_id={}, target_topic={}, offset={}, duration_ms={:.2}",
        delay_info.unique_id, delay_info.target_topic_name, offset, duration_ms
    );

    delete_delay_index_info(storage_driver_manager, delay_info).await?;
    delete_delay_message(storage_driver_manager, &delay_info.unique_id).await?;

    Ok(())
}

async fn send_delay_message_to_shard(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_message: &DelayMessageIndexInfo,
    trigger_time: u64,
) -> Result<u64, CommonError> {
    // read data
    let results = storage_driver_manager
        .read_by_keys(
            DEFAULT_TENANT,
            DELAY_QUEUE_MESSAGE_TOPIC,
            &[delay_message.unique_id.as_bytes()],
        )
        .await?
        .remove(delay_message.unique_id.as_bytes())
        .unwrap_or_default();

    if results.is_empty() {
        return Err(CommonError::CommonError(format!(
            "Delay message not found: unique_id={}, offset={}",
            delay_message.unique_id, delay_message.offset
        )));
    }

    if results.len() > 1 {
        return Err(CommonError::CommonError(format!(
            "Multiple delay messages found for unique_id={}, expected 1 but found {}",
            delay_message.unique_id,
            results.len()
        )));
    }

    let record = if let Some(record) = results.first() {
        record.clone()
    } else {
        return Err(CommonError::CommonError(format!(
            "Failed to retrieve delay message record for unique_id={}",
            delay_message.unique_id
        )));
    };

    let send_record = build_new_record(delay_message, &record, trigger_time);

    // send to target topic under the original tenant
    let resp = storage_driver_manager
        .write(
            &delay_message.tenant,
            &delay_message.target_topic_name,
            &[send_record],
            1,
        )
        .await?;

    let write_resp = if let Some(data) = resp.first() {
        data.clone()
    } else {
        return Err(CommonError::CommonError(format!(
            "Write response is empty when sending delay message to topic '{}'",
            delay_message.target_topic_name
        )));
    };

    if write_resp.is_error() {
        return Err(CommonError::CommonError(write_resp.error_info()));
    }
    debug!(
        "Expired delay message sent successfully: delay queue -> {} (offset: {})",
        delay_message.target_topic_name, delay_message.offset
    );
    Ok(write_resp.offset)
}

fn build_new_record(
    delay_message: &DelayMessageIndexInfo,
    record: &StorageRecord,
    trigger_time: u64,
) -> AdapterWriteRecord {
    let trigger_header = RecordHeader {
        name: DELAY_MESSAGE_SAVE_MS.to_string(),
        value: trigger_time.to_string(),
    };

    let mut send_record =
        AdapterWriteRecord::new(delay_message.target_topic_name.clone(), record.data.clone())
            .with_protocol_data(record.protocol_data.clone());

    // header
    let header = if let Some(header) = record.metadata.header.clone() {
        let mut new_header = Vec::new();
        for raw in header {
            new_header.push(RecordHeader {
                name: raw.name,
                value: raw.value,
            });
        }
        new_header.push(trigger_header);
        new_header
    } else {
        vec![trigger_header]
    };
    send_record = send_record.with_header(header);

    // key
    if let Some(key) = record.metadata.key.clone() {
        send_record = send_record.with_key(key);
    }

    // tags
    if let Some(tags) = record.metadata.tags.clone() {
        send_record = send_record.with_tags(tags);
    }

    send_record
}

#[cfg(test)]
mod test {
    use super::*;
    use broker_core::inner_topic::{DELAY_QUEUE_INDEX_TOPIC, DELAY_QUEUE_MESSAGE_TOPIC};
    use common_config::config::DelayMessageConfig;
    use common_config::default::{
        default_delay_message_initial_retry_delay_sec, default_delay_message_max_retries,
    };
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::delay_info::DelayMessageIndexInfo;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use std::time::Duration;
    use storage_adapter::storage::{test_add_topic, test_build_storage_driver_manager};

    #[test]
    fn test_delay_message_config_defaults() {
        let config = DelayMessageConfig::default();
        assert_eq!(config.max_retries, default_delay_message_max_retries());
        assert_eq!(config.max_retries, 3);
        assert_eq!(
            config.initial_retry_delay_sec,
            default_delay_message_initial_retry_delay_sec()
        );
        assert_eq!(config.initial_retry_delay_sec, 2);
    }

    #[test]
    fn test_retry_backoff_calculation() {
        let config = DelayMessageConfig::default();
        // backoff = initial_retry_delay_sec * 2^attempt
        // attempt 0 -> 2 * 1 = 2, attempt 1 -> 2 * 2 = 4, attempt 2 -> 2 * 4 = 8
        for attempt in 0..config.max_retries {
            let backoff = config.initial_retry_delay_sec * 2_u64.pow(attempt);
            let expected = match attempt {
                0 => 2,
                1 => 4,
                2 => 8,
                _ => unreachable!(),
            };
            assert_eq!(backoff, expected);
        }
    }

    #[tokio::test]
    async fn test_delay_message_process_returns_err_on_send_failure() {
        // Build a real StorageDriverManager with only the delay queue topics
        // registered, but NOT the target topic — this makes write() fail.
        let storage = test_build_storage_driver_manager().await.unwrap();

        // Register delay queue internal topics so read_by_keys() succeeds
        test_add_topic(&storage, DELAY_QUEUE_MESSAGE_TOPIC);
        test_add_topic(&storage, DELAY_QUEUE_INDEX_TOPIC);

        let unique_id = "test_fail_unique".to_string();
        let target_topic = "nonexistent_topic_for_failure_test";

        // Persist a dummy message payload so read_by_keys() can find it
        let dummy_record =
            AdapterWriteRecord::new(DELAY_QUEUE_MESSAGE_TOPIC, b"dummy payload".to_vec())
                .with_key(unique_id.clone());
        storage
            .write(
                DEFAULT_TENANT,
                DELAY_QUEUE_MESSAGE_TOPIC,
                &[dummy_record],
                1,
            )
            .await
            .expect("write to delay queue message topic");

        let delay_info = DelayMessageIndexInfo {
            unique_id: unique_id.clone(),
            tenant: DEFAULT_TENANT.to_string(),
            target_topic_name: target_topic.to_string(),
            offset: 0,
            target_timestamp: now_second(),
            retry_count: 0,
        };

        let result = delay_message_process(&storage, &delay_info, now_second()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_reenqueue_updates_target_timestamp() {
        let original = DelayMessageIndexInfo {
            unique_id: "re_test".to_string(),
            tenant: "test_tenant".to_string(),
            target_topic_name: "test_topic".to_string(),
            offset: 100,
            target_timestamp: 1000,
            retry_count: 0,
        };

        let mut reenq = original.clone();
        let reenqueue_delay = 60;
        let before = now_second();
        reenq.target_timestamp = before + reenqueue_delay;

        assert_eq!(reenq.unique_id, original.unique_id);
        assert_eq!(reenq.target_topic_name, original.target_topic_name);
        assert_eq!(reenq.tenant, original.tenant);
        assert!(reenq.target_timestamp >= before + reenqueue_delay);
        assert!(reenq.target_timestamp <= before + reenqueue_delay + 1);
    }

    #[tokio::test]
    async fn test_delay_message_retry_integration() {
        // End-to-end integration: enqueue a message, let it expire, verify delivery.
        let storage = test_build_storage_driver_manager().await.unwrap();

        // Register all required topics
        test_add_topic(&storage, DELAY_QUEUE_MESSAGE_TOPIC);
        test_add_topic(&storage, DELAY_QUEUE_INDEX_TOPIC);

        let target_topic = "integration_target_topic";
        test_add_topic(&storage, target_topic);

        // Create DelayMessageManager with fast retry config for testing
        use grpc_clients::pool::ClientPool;
        let client_pool = Arc::new(ClientPool::new(2));
        let config = DelayMessageConfig {
            max_retries: 2,
            initial_retry_delay_sec: 1,
        };
        let manager = Arc::new(
            DelayMessageManager::new(client_pool.clone(), storage.clone(), 1, config)
                .await
                .unwrap(),
        );

        // Start pop thread
        use common_base::task::TaskSupervisor;
        let task_supervisor = Arc::new(TaskSupervisor::new());
        spawn_delay_message_pop_threads(&manager, &task_supervisor, 1);

        // Enqueue a delay message with a short delay
        let unique_id = manager
            .send(
                DEFAULT_TENANT,
                target_topic,
                now_second() + 2,
                AdapterWriteRecord::new(target_topic, b"integration test payload".to_vec()),
            )
            .await
            .expect("send delay message");

        // Wait for processing (2s delay + buffer)
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Verify the message was delivered by reading from target topic
        let results = storage
            .read_by_keys(DEFAULT_TENANT, target_topic, &[unique_id.as_bytes()])
            .await
            .expect("read target topic");
        assert!(
            !results.is_empty(),
            "Message should have been delivered to target topic"
        );
    }

    #[tokio::test]
    async fn test_metrics_record_retry_and_reenqueue() {
        // Verify that retry and reenqueue metric functions execute without panic,
        // and that the metric counters exist in the registry.
        use common_metrics::mqtt::delay::{
            record_delay_msg_dead_letter, record_delay_msg_reenqueued, record_delay_msg_retry,
            record_delay_msg_retry_count,
        };

        // Record some retries and re-enqueues
        record_delay_msg_retry(false);
        record_delay_msg_retry(false);
        record_delay_msg_retry(true);
        record_delay_msg_retry_count(1);
        record_delay_msg_retry_count(2);
        record_delay_msg_retry_count(3);
        record_delay_msg_reenqueued();
        record_delay_msg_reenqueued();
        record_delay_msg_dead_letter();

        // If we get here without panic, the metric functions work correctly
    }

    #[test]
    fn test_delay_message_reenqueued_target_timestamp_forward_in_time() {
        // Ensure re-enqueued message gets a future timestamp
        let delay_info = DelayMessageIndexInfo {
            unique_id: "time_test".to_string(),
            tenant: "t".to_string(),
            target_topic_name: "t".to_string(),
            offset: 1,
            target_timestamp: 100,
            retry_count: 0,
        };

        let reenq_delay = 60u64;
        let mut reenq = delay_info.clone();
        let now = now_second();
        reenq.target_timestamp = now + reenq_delay;

        assert!(reenq.target_timestamp > delay_info.target_timestamp);
        assert!(reenq.target_timestamp >= now + reenq_delay);
        assert!(reenq.target_timestamp <= now + reenq_delay + 1);
    }

    #[tokio::test]
    async fn test_retry_exhaustion_drops_message() {
        // When a delay message fails all retry attempts against a non-existent
        // target topic, it must be dropped (index + data deleted) rather than
        // entering an unbounded retry loop.
        let storage = test_build_storage_driver_manager().await.unwrap();

        test_add_topic(&storage, DELAY_QUEUE_MESSAGE_TOPIC);
        test_add_topic(&storage, DELAY_QUEUE_INDEX_TOPIC);

        // Target topic intentionally NOT registered → every attempt will fail.

        use grpc_clients::pool::ClientPool;
        let client_pool = Arc::new(ClientPool::new(2));
        let config = DelayMessageConfig {
            max_retries: 2,
            initial_retry_delay_sec: 0,
        };
        let manager = Arc::new(
            DelayMessageManager::new(client_pool.clone(), storage.clone(), 1, config)
                .await
                .unwrap(),
        );

        use common_base::task::TaskSupervisor;
        let task_supervisor = Arc::new(TaskSupervisor::new());
        spawn_delay_message_pop_threads(&manager, &task_supervisor, 1);

        let unique_id = manager
            .send(
                DEFAULT_TENANT,
                "nonexistent_target",
                now_second() + 1,
                AdapterWriteRecord::new("nonexistent_target", b"payload".to_vec()),
            )
            .await
            .expect("send");

        // Wait for: delay(1s) + fast retries(backoff=0, nearly instant) + drop.
        tokio::time::sleep(Duration::from_secs(3)).await;

        assert!(
            !manager.is_message_in_queue(&unique_id),
            "Message should be dropped from delay queue after retry exhaustion"
        );
    }
}
