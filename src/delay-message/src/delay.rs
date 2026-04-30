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

use broker_core::inner_topic::{DELAY_QUEUE_INDEX_TOPIC, DELAY_QUEUE_MESSAGE_TOPIC};
use common_base::error::common::CommonError;
use common_base::utils::serialize::serialize;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::tenant::DEFAULT_TENANT;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tracing::debug;

pub(crate) async fn save_delay_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_message_id: &str,
    mut data: AdapterWriteRecord,
) -> Result<u64, CommonError> {
    data.key = Some(delay_message_id.to_string());
    let result = storage_driver_manager
        .write(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC, &[data])
        .await?;

    let resp = if let Some(row) = result.first() {
        row.clone()
    } else {
        return Err(CommonError::CommonError(format!(
            "Write response is empty when saving delay message to topic '{}'",
            DELAY_QUEUE_MESSAGE_TOPIC
        )));
    };

    if resp.is_error() {
        return Err(CommonError::CommonError(resp.error_info()));
    }

    debug!(
        "Delay message persisted to shard {} at offset {}",
        DELAY_QUEUE_MESSAGE_TOPIC, resp.offset
    );

    Ok(resp.offset)
}

pub(crate) async fn delete_delay_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    unique_id: &str,
) -> Result<(), CommonError> {
    storage_driver_manager
        .delete_by_key(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC, unique_id)
        .await?;
    debug!(
        "Deleted delay message: shard={}, unique_id={}",
        DELAY_QUEUE_MESSAGE_TOPIC, unique_id
    );
    Ok(())
}

pub async fn save_delay_index_info(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    let data = serialize(&delay_info)?;
    let record = AdapterWriteRecord::new(DELAY_QUEUE_INDEX_TOPIC, data)
        .with_key(delay_info.unique_id.clone());

    let result = storage_driver_manager
        .write(DEFAULT_TENANT, DELAY_QUEUE_INDEX_TOPIC, &[record])
        .await?;

    let resp = if let Some(row) = result.first() {
        row.clone()
    } else {
        return Err(CommonError::CommonError(format!(
            "Write response is empty when saving delay index info (unique_id={}) to topic '{}'",
            delay_info.unique_id, DELAY_QUEUE_INDEX_TOPIC
        )));
    };

    if resp.is_error() {
        return Err(CommonError::CommonError(resp.error_info()));
    }
    Ok(())
}

pub(crate) async fn delete_delay_index_info(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    storage_driver_manager
        .delete_by_key(
            DEFAULT_TENANT,
            DELAY_QUEUE_INDEX_TOPIC,
            &delay_info.unique_id,
        )
        .await?;
    debug!(
        "Deleted delay index info: unique_id={}",
        delay_info.unique_id
    );
    Ok(())
}

#[cfg(test)]
mod test {}
