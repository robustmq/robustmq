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
use metadata_struct::connector::FailureHandlingStrategy;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::time::sleep;
use tracing::{debug, error};

use crate::storage::message::MessageStorage;

/// Record written to the dead letter queue when a connector fails to deliver messages.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeadLetterRecord {
    pub connector_name: String,
    pub source_topic: String,
    pub error_message: String,
    pub retry_times: u32,
    pub original_key: Option<String>,
    pub original_data: Vec<u8>,
    pub original_timestamp: u64,
    pub dead_letter_timestamp: u64,
}

pub struct FailureContext<'a> {
    pub storage_driver_manager: &'a Arc<StorageDriverManager>,
    pub connector_name: &'a str,
    pub source_topic: &'a str,
    pub error_message: &'a str,
    pub records: &'a [AdapterWriteRecord],
}

pub async fn failure_message_process(
    strategy: FailureHandlingStrategy,
    retry_times: u32,
    context: &FailureContext<'_>,
) -> bool {
    match strategy {
        FailureHandlingStrategy::Discard => true,
        FailureHandlingStrategy::DiscardAfterRetry(strategy) => {
            if retry_times < strategy.retry_total_times {
                sleep(Duration::from_millis(strategy.wait_time_ms)).await;
                return false;
            }
            true
        }
        FailureHandlingStrategy::DeadMessageQueue(strategy) => {
            if retry_times < strategy.retry_total_times {
                sleep(Duration::from_millis(strategy.wait_time_ms)).await;
                return false;
            }
            if let Err(e) =
                send_to_dead_letter_queue(&strategy.topic_name, retry_times, context).await
            {
                error!(
                    "Failed to write dead letter queue for connector '{}', will retry. reason: {}",
                    context.connector_name, e
                );
                sleep(Duration::from_millis(strategy.wait_time_ms)).await;
                return false;
            }
            true
        }
    }
}

async fn send_to_dead_letter_queue(
    dlq_topic: &str,
    retry_times: u32,
    context: &FailureContext<'_>,
) -> Result<(), CommonError> {
    let now = now_second();
    let mut dlq_records = Vec::with_capacity(context.records.len());

    for record in context.records {
        let dead_letter = DeadLetterRecord {
            connector_name: context.connector_name.to_string(),
            source_topic: context.source_topic.to_string(),
            error_message: context.error_message.to_string(),
            retry_times,
            original_key: record.key.clone(),
            original_data: record.data.to_vec(),
            original_timestamp: record.timestamp,
            dead_letter_timestamp: now,
        };

        let data = serde_json::to_vec(&dead_letter).map_err(|e| {
            CommonError::CommonError(format!(
                "Failed to serialize dead letter record for connector '{}': {}",
                context.connector_name, e
            ))
        })?;
        let mut dlq_record = AdapterWriteRecord::from_bytes(data);
        if let Some(key) = &record.key {
            dlq_record.set_key(key.clone());
        }
        dlq_records.push(dlq_record);
    }

    if dlq_records.is_empty() {
        return Ok(());
    }

    let message_storage = MessageStorage::new(context.storage_driver_manager.clone());
    let offsets = message_storage
        .append_topic_message(dlq_topic, dlq_records)
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
