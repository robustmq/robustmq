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

use crate::storage::message::MessageStorage;
use common_base::error::common::CommonError;
use common_metrics::mqtt::connector::{
    record_connector_dlq_messages, record_connector_messages_discarded, record_connector_retry,
};
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::{connector::FailureHandlingStrategy, storage::record::StorageRecord};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::time::sleep;
use tracing::{debug, error};

/// Record written to the dead letter queue when a connector fails to deliver messages.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeadLetterRecord {
    pub connector_name: String,
    pub source_topic: String,
    pub error_message: String,
    pub retry_times: u32,
    pub record: StorageRecord,
}

pub struct FailureRecordInfo {
    pub tenant: String,
    pub connector_name: String,
    pub connector_type: String,
    pub source_topic: String,
    pub error_message: String,
    pub records: Vec<StorageRecord>,
}

pub async fn failure_message_process(
    storage_driver_manager: &Arc<StorageDriverManager>,
    strategy: &FailureHandlingStrategy,
    retry_times: u32,
    context: &FailureRecordInfo,
) -> bool {
    match strategy {
        FailureHandlingStrategy::Discard => {
            record_connector_messages_discarded(
                &context.tenant,
                context.connector_type.to_string(),
                context.connector_name.to_string(),
                "discard",
                context.records.len() as u64,
            );
            true
        }
        FailureHandlingStrategy::DiscardAfterRetry(discard_strategy) => {
            if retry_times < discard_strategy.retry_total_times {
                record_connector_retry(
                    &context.tenant,
                    context.connector_type.to_string(),
                    context.connector_name.to_string(),
                    "discard_after_retry",
                );
                sleep(Duration::from_millis(discard_strategy.wait_time_ms)).await;
                return false;
            }
            record_connector_messages_discarded(
                &context.tenant,
                context.connector_type.to_string(),
                context.connector_name.to_string(),
                "discard_after_retry",
                context.records.len() as u64,
            );
            true
        }
        FailureHandlingStrategy::DeadMessageQueue(dlq_strategy) => {
            if retry_times < dlq_strategy.retry_total_times {
                record_connector_retry(
                    &context.tenant,
                    context.connector_type.to_string(),
                    context.connector_name.to_string(),
                    "dead_message_queue",
                );
                sleep(Duration::from_millis(dlq_strategy.wait_time_ms)).await;
                return false;
            }
            if let Err(e) = send_to_dead_letter_queue(
                storage_driver_manager,
                &dlq_strategy.tenant,
                &dlq_strategy.topic_name,
                retry_times,
                context,
            )
            .await
            {
                record_connector_dlq_messages(
                    &context.tenant,
                    context.connector_type.to_string(),
                    context.connector_name.to_string(),
                    "failure",
                    context.records.len() as u64,
                );
                error!(
                    "Failed to write dead letter queue for connector '{}', will retry. reason: {}",
                    context.connector_name, e
                );
                sleep(Duration::from_millis(dlq_strategy.wait_time_ms)).await;
                return false;
            }
            record_connector_dlq_messages(
                &context.tenant,
                context.connector_type.to_string(),
                context.connector_name.to_string(),
                "success",
                context.records.len() as u64,
            );
            true
        }
    }
}

async fn send_to_dead_letter_queue(
    storage_driver_manager: &Arc<StorageDriverManager>,
    tenant: &str,
    dlq_topic: &str,
    retry_times: u32,
    context: &FailureRecordInfo,
) -> Result<(), CommonError> {
    let mut dlq_records = Vec::with_capacity(context.records.len());

    for record in context.records.iter() {
        let dead_letter = DeadLetterRecord {
            connector_name: context.connector_name.to_string(),
            source_topic: context.source_topic.to_string(),
            error_message: context.error_message.to_string(),
            retry_times,
            record: record.clone(),
        };

        let data = serde_json::to_vec(&dead_letter).map_err(|e| {
            CommonError::CommonError(format!(
                "Failed to serialize dead letter record for connector '{}': {}",
                context.connector_name, e
            ))
        })?;
        let dlq_record = AdapterWriteRecord::new(dlq_topic, data);
        dlq_records.push(dlq_record);
    }

    if dlq_records.is_empty() {
        return Ok(());
    }

    let message_storage = MessageStorage::new(storage_driver_manager.clone());
    let offsets = message_storage
        .append_topic_message(tenant, dlq_topic, dlq_records)
        .await?;
    debug!(
        "Wrote {} dead letter records to topic '{}' for connector '{}', offsets: {:?}",
        context.records.len(),
        dlq_topic,
        context.connector_name,
        offsets
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::connector::{
        DeadMessageQueueStrategy, DiscardAfterRetryStrategy, FailureHandlingStrategy,
    };
    use storage_adapter::storage::test_build_storage_driver_manager;

    fn failure_context() -> FailureRecordInfo {
        FailureRecordInfo {
            tenant: "tenant".to_string(),
            connector_name: "connector".to_string(),
            connector_type: "file".to_string(),
            source_topic: "source".to_string(),
            error_message: "send failed".to_string(),
            records: Vec::new(),
        }
    }

    #[tokio::test]
    async fn discard_finishes_without_retry() {
        let storage = test_build_storage_driver_manager().await.unwrap();

        assert!(
            failure_message_process(
                &storage,
                &FailureHandlingStrategy::Discard,
                0,
                &failure_context(),
            )
            .await
        );
    }

    #[tokio::test]
    async fn discard_after_retry_waits_until_retry_limit() {
        let storage = test_build_storage_driver_manager().await.unwrap();
        let strategy = FailureHandlingStrategy::DiscardAfterRetry(DiscardAfterRetryStrategy {
            retry_total_times: 2,
            wait_time_ms: 0,
        });

        assert!(!failure_message_process(&storage, &strategy, 0, &failure_context()).await);
        assert!(!failure_message_process(&storage, &strategy, 1, &failure_context()).await);
        assert!(failure_message_process(&storage, &strategy, 2, &failure_context()).await);
    }

    #[tokio::test]
    async fn empty_dead_letter_batch_finishes_without_storage_write() {
        let storage = test_build_storage_driver_manager().await.unwrap();
        let strategy = FailureHandlingStrategy::DeadMessageQueue(DeadMessageQueueStrategy {
            tenant: "dlq-tenant".to_string(),
            topic_name: "dead-letter".to_string(),
            retry_total_times: 0,
            wait_time_ms: 0,
        });

        assert!(failure_message_process(&storage, &strategy, 0, &failure_context()).await);
    }

    #[test]
    fn dead_letter_record_round_trips_through_json() {
        let record = DeadLetterRecord {
            connector_name: "connector".to_string(),
            source_topic: "source".to_string(),
            error_message: "send failed".to_string(),
            retry_times: 3,
            record: StorageRecord {
                metadata: Default::default(),
                protocol_data: None,
                data: bytes::Bytes::from_static(b"payload"),
            },
        };

        let encoded = serde_json::to_vec(&record).unwrap();
        let decoded: DeadLetterRecord = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(decoded.connector_name, "connector");
        assert_eq!(decoded.retry_times, 3);
        assert_eq!(decoded.record.data, bytes::Bytes::from_static(b"payload"));
    }
}
