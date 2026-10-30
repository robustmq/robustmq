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

use crate::manager::DelayMessageManager;
use crate::pop::process_expired_delay_message;
use broker_core::inner_topic::DELAY_QUEUE_INDEX_TOPIC;
use common_base::tools::now_second;
use common_base::utils::serialize::{self};
use common_metrics::mqtt::delay::{record_delay_msg_recover, record_delay_msg_recover_expired};
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::tenant::DEFAULT_TENANT;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

const MAX_READ_RETRY: u32 = 3;
const PROGRESS_LOG_INTERVAL: u64 = 1000;

enum ReadBatch {
    Data(Vec<metadata_struct::storage::record::StorageRecord>),
    Retry,
    Abort,
}

pub(crate) async fn recover_delay_queue(delay_message_manager: &Arc<DelayMessageManager>) {
    info!("Starting delay queue recovery from persistent storage");

    let read_config = AdapterReadConfig {
        max_record_num: 100,
        max_size: 10 * 1024 * 1024,
    };

    let mut offsets = HashMap::new();
    let mut total_num = 0;
    let mut last_progress_log = 0;
    let mut retry_count = 0;

    loop {
        let data = match read_delay_index_batch(
            delay_message_manager,
            &offsets,
            &read_config,
            &mut retry_count,
        )
        .await
        {
            ReadBatch::Data(data) => data,
            ReadBatch::Retry => continue,
            ReadBatch::Abort => return,
        };

        if data.is_empty() {
            break;
        }

        for record in &data {
            if process_delay_index_record(delay_message_manager, record, &mut total_num).await
                && total_num - last_progress_log >= PROGRESS_LOG_INTERVAL
            {
                info!(
                    "Delay queue recovery progress: {} messages recovered",
                    total_num
                );
                last_progress_log = total_num;
            }
        }

        update_offsets_from_records(&data, &mut offsets);
    }

    info!(
        "Delay queue recovery completed. Total messages recovered: {}",
        total_num
    );
}

async fn read_delay_index_batch(
    delay_message_manager: &Arc<DelayMessageManager>,
    offsets: &HashMap<String, u64>,
    read_config: &AdapterReadConfig,
    retry_count: &mut u32,
) -> ReadBatch {
    match delay_message_manager
        .storage_driver_manager
        .read_by_offset(
            DEFAULT_TENANT,
            DELAY_QUEUE_INDEX_TOPIC,
            offsets,
            read_config,
        )
        .await
    {
        Ok(data) => {
            *retry_count = 0;
            ReadBatch::Data(data)
        }
        Err(e) => {
            *retry_count += 1;
            error!(
                "Reading shard {} failed (attempt {}/{}): {:?}",
                DELAY_QUEUE_INDEX_TOPIC, retry_count, MAX_READ_RETRY, e
            );

            if *retry_count >= MAX_READ_RETRY {
                error!(
                    "Failed to recover delay queue after {} retries, aborting recovery",
                    MAX_READ_RETRY
                );
                return ReadBatch::Abort;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
            ReadBatch::Retry
        }
    }
}

async fn process_delay_index_record(
    delay_message_manager: &Arc<DelayMessageManager>,
    record: &metadata_struct::storage::record::StorageRecord,
    total_num: &mut u64,
) -> bool {
    let delay_info = match serialize::deserialize::<DelayMessageIndexInfo>(&record.data) {
        Ok(info) => info,
        Err(e) => {
            error!(
                "Failed to deserialize delay index at shard={}, offset={}: {:?}",
                DELAY_QUEUE_INDEX_TOPIC, record.metadata.offset, e
            );
            return false;
        }
    };

    let now = now_second();
    if delay_info.target_timestamp < now {
        handle_expired_delay_message(delay_message_manager, delay_info, now).await;
        return false;
    }

    if let Err(e) = delay_message_manager.send_to_delay_queue(&delay_info).await {
        error!(
            "Failed to enqueue delay message during recovery: unique_id={}, error={}",
            delay_info.unique_id, e
        );
        return false;
    }
    record_delay_msg_recover();
    *total_num += 1;
    true
}

async fn handle_expired_delay_message(
    delay_message_manager: &Arc<DelayMessageManager>,
    delay_info: DelayMessageIndexInfo,
    now: u64,
) {
    record_delay_msg_recover_expired();
    warn!(
        "Delay message expired during recovery, sending immediately. \
         offset: {}, target: {}, expired by: {}s",
        delay_info.offset,
        delay_info.target_topic_name,
        now - delay_info.target_timestamp
    );

    tokio::spawn(process_expired_delay_message(
        delay_message_manager.clone(),
        delay_info,
    ));
}

fn update_offsets_from_records(
    data: &[metadata_struct::storage::record::StorageRecord],
    offsets: &mut HashMap<String, u64>,
) {
    for record in data {
        let current_offset = offsets.get(&record.metadata.shard).copied().unwrap_or(0);
        offsets.insert(
            record.metadata.shard.clone(),
            current_offset.max(record.metadata.offset + 1),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::Bytes;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
    use storage_adapter::storage::{test_add_topic, test_build_storage_driver_manager};

    async fn test_manager() -> Arc<DelayMessageManager> {
        Arc::new(
            DelayMessageManager::new(
                Arc::new(ClientPool::new(1)),
                test_build_storage_driver_manager().await.unwrap(),
                1,
                common_config::config::DelayMessageConfig::default(),
            )
            .await
            .unwrap(),
        )
    }

    fn record(offset: u64, shard: &str, data: Vec<u8>) -> StorageRecord {
        StorageRecord {
            metadata: StorageRecordMetadata {
                offset,
                shard: shard.to_string(),
                ..Default::default()
            },
            protocol_data: None,
            data: Bytes::from(data),
        }
    }

    #[test]
    fn updates_offsets_per_shard_using_highest_next_offset() {
        let records = vec![
            record(4, "shard-a", vec![]),
            record(2, "shard-b", vec![]),
            record(1, "shard-a", vec![]),
            record(9, "shard-b", vec![]),
        ];
        let mut offsets = HashMap::from([(String::from("shard-a"), 7)]);
        update_offsets_from_records(&records, &mut offsets);
        assert_eq!(offsets.get("shard-a"), Some(&7));
        assert_eq!(offsets.get("shard-b"), Some(&10));
    }

    #[tokio::test]
    async fn read_batch_resets_retry_count_on_success() {
        let manager = test_manager().await;
        test_add_topic(&manager.storage_driver_manager, DELAY_QUEUE_INDEX_TOPIC);
        let mut retry_count = 2;
        let result = read_delay_index_batch(
            &manager,
            &HashMap::new(),
            &AdapterReadConfig::default(),
            &mut retry_count,
        )
        .await;
        assert!(matches!(result, ReadBatch::Data(_)));
        assert_eq!(retry_count, 0);
    }

    #[tokio::test]
    async fn read_batch_aborts_after_max_retries() {
        let manager = test_manager().await;
        let mut retry_count = MAX_READ_RETRY - 1;
        let result = read_delay_index_batch(
            &manager,
            &HashMap::new(),
            &AdapterReadConfig::default(),
            &mut retry_count,
        )
        .await;
        assert!(matches!(result, ReadBatch::Abort));
        assert_eq!(retry_count, MAX_READ_RETRY);
    }

    #[tokio::test]
    async fn read_batch_returns_retry_before_abort() {
        let manager = test_manager().await;
        let mut retry_count = 0;
        let result = read_delay_index_batch(
            &manager,
            &HashMap::new(),
            &AdapterReadConfig::default(),
            &mut retry_count,
        )
        .await;

        assert!(matches!(result, ReadBatch::Retry));
        assert_eq!(retry_count, 1);
    }

    #[tokio::test]
    async fn invalid_index_record_is_ignored() {
        let manager = test_manager().await;
        let mut total = 0;
        assert!(
            !process_delay_index_record(&manager, &record(3, "shard", vec![1, 2, 3]), &mut total)
                .await
        );
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn future_index_record_requires_registered_pop_channel() {
        let manager = test_manager().await;
        let info = DelayMessageIndexInfo {
            unique_id: "future-recovery".to_string(),
            tenant: DEFAULT_TENANT.to_string(),
            target_topic_name: "target".to_string(),
            offset: 3,
            target_timestamp: now_second() + 3600,
            retry_count: 0,
        };
        let data = common_base::utils::serialize::serialize(&info).unwrap();
        let mut total = 0;
        assert!(!process_delay_index_record(&manager, &record(3, "shard", data), &mut total).await);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn recovery_with_empty_index_topic_completes() {
        let manager = test_manager().await;
        test_add_topic(&manager.storage_driver_manager, DELAY_QUEUE_INDEX_TOPIC);
        recover_delay_queue(&manager).await;
    }

    #[tokio::test]
    async fn expired_message_recovery_requeues_failed_delivery_for_retry() {
        use crate::delay::{save_delay_index_info, save_delay_message};
        use crate::pop::spawn_delay_message_pop_threads;
        use broker_core::inner_topic::DELAY_QUEUE_MESSAGE_TOPIC;
        use common_base::task::TaskSupervisor;
        use common_config::config::DelayMessageConfig;

        let storage = test_build_storage_driver_manager().await.unwrap();
        test_add_topic(&storage, DELAY_QUEUE_MESSAGE_TOPIC);
        test_add_topic(&storage, DELAY_QUEUE_INDEX_TOPIC);

        let manager = Arc::new(
            DelayMessageManager::new(
                Arc::new(ClientPool::new(1)),
                storage.clone(),
                1,
                DelayMessageConfig {
                    max_retries: 2,
                    initial_retry_delay_sec: 60,
                },
            )
            .await
            .unwrap(),
        );
        let supervisor = Arc::new(TaskSupervisor::new());
        spawn_delay_message_pop_threads(&manager, &supervisor, 1);

        let unique_id = "expired-recovery-retry";
        let offset = save_delay_message(
            &storage,
            unique_id,
            AdapterWriteRecord::new(DELAY_QUEUE_MESSAGE_TOPIC, b"payload".to_vec()),
            1,
        )
        .await
        .unwrap();
        let delay_info = DelayMessageIndexInfo {
            unique_id: unique_id.to_string(),
            tenant: DEFAULT_TENANT.to_string(),
            target_topic_name: "missing-target".to_string(),
            offset,
            target_timestamp: now_second().saturating_sub(1),
            retry_count: 0,
        };
        save_delay_index_info(&storage, &delay_info, 1)
            .await
            .unwrap();

        let index_record = record(
            0,
            DELAY_QUEUE_INDEX_TOPIC,
            serialize::serialize(&delay_info).unwrap(),
        );
        let mut recovered = 0;
        assert!(!process_delay_index_record(&manager, &index_record, &mut recovered).await);

        tokio::time::timeout(Duration::from_secs(2), async {
            while !manager.is_message_in_queue(unique_id) {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("failed recovery delivery should be re-enqueued for retry");

        let persisted_index = storage
            .read_by_keys(
                DEFAULT_TENANT,
                DELAY_QUEUE_INDEX_TOPIC,
                &[unique_id.as_bytes()],
            )
            .await
            .unwrap();
        let persisted_info: DelayMessageIndexInfo = serialize::deserialize(
            &persisted_index
                .get(unique_id.as_bytes())
                .expect("retry index should remain persisted")
                .first()
                .expect("retry index should contain a record")
                .data,
        )
        .unwrap();
        assert_eq!(persisted_info.retry_count, 1);

        manager.stop().await.unwrap();
    }
}
