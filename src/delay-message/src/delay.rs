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
    acks: i8,
) -> Result<u64, CommonError> {
    data.key = Some(delay_message_id.to_string().into());
    let result = storage_driver_manager
        .write(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC, &[data], acks)
        .await?;

    let resp = if let Some(row) = result.first() {
        row
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
        .delete_by_keys(
            DEFAULT_TENANT,
            DELAY_QUEUE_MESSAGE_TOPIC,
            &[unique_id.as_bytes()],
        )
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
    acks: i8,
) -> Result<(), CommonError> {
    let data = serialize(&delay_info)?;
    let record = AdapterWriteRecord::new(DELAY_QUEUE_INDEX_TOPIC, data)
        .with_key(delay_info.unique_id.clone());

    let result = storage_driver_manager
        .write(DEFAULT_TENANT, DELAY_QUEUE_INDEX_TOPIC, &[record], acks)
        .await?;

    let resp = if let Some(row) = result.first() {
        row
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
        .delete_by_keys(
            DEFAULT_TENANT,
            DELAY_QUEUE_INDEX_TOPIC,
            &[delay_info.unique_id.as_bytes()],
        )
        .await?;
    debug!(
        "Deleted delay index info: unique_id={}",
        delay_info.unique_id
    );
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use broker_core::inner_topic::{DELAY_QUEUE_INDEX_TOPIC, DELAY_QUEUE_MESSAGE_TOPIC};
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use storage_adapter::storage::{test_add_topic, test_build_storage_driver_manager};

    fn delay_info(unique_id: &str) -> DelayMessageIndexInfo {
        DelayMessageIndexInfo {
            unique_id: unique_id.to_string(),
            tenant: DEFAULT_TENANT.to_string(),
            target_topic_name: "target".to_string(),
            offset: 0,
            target_timestamp: 42,
            retry_count: 0,
        }
    }

    #[tokio::test]
    async fn saves_and_deletes_delay_message_and_index() {
        let storage = test_build_storage_driver_manager().await.unwrap();
        test_add_topic(&storage, DELAY_QUEUE_MESSAGE_TOPIC);
        test_add_topic(&storage, DELAY_QUEUE_INDEX_TOPIC);

        let unique_id = "delay-storage-roundtrip";
        let _offset = save_delay_message(
            &storage,
            unique_id,
            AdapterWriteRecord::new("source", b"payload".to_vec()),
            1,
        )
        .await
        .unwrap();

        let info = delay_info(unique_id);
        save_delay_index_info(&storage, &info, 1).await.unwrap();

        let message = storage
            .read_by_keys(
                DEFAULT_TENANT,
                DELAY_QUEUE_MESSAGE_TOPIC,
                &[unique_id.as_bytes()],
            )
            .await
            .unwrap();
        assert_eq!(message.get(unique_id.as_bytes()).unwrap().len(), 1);
        let index = storage
            .read_by_keys(
                DEFAULT_TENANT,
                DELAY_QUEUE_INDEX_TOPIC,
                &[unique_id.as_bytes()],
            )
            .await
            .unwrap();
        assert_eq!(index.get(unique_id.as_bytes()).unwrap().len(), 1);

        delete_delay_index_info(&storage, &info).await.unwrap();
        delete_delay_message(&storage, unique_id).await.unwrap();
        let after_message = storage
            .read_by_keys(
                DEFAULT_TENANT,
                DELAY_QUEUE_MESSAGE_TOPIC,
                &[unique_id.as_bytes()],
            )
            .await
            .unwrap();
        assert!(after_message
            .get(unique_id.as_bytes())
            .is_none_or(Vec::is_empty));
        let after_index = storage
            .read_by_keys(
                DEFAULT_TENANT,
                DELAY_QUEUE_INDEX_TOPIC,
                &[unique_id.as_bytes()],
            )
            .await
            .unwrap();
        assert!(after_index
            .get(unique_id.as_bytes())
            .is_none_or(Vec::is_empty));
    }

    #[tokio::test]
    async fn saving_delay_message_requires_registered_topic() {
        let storage = test_build_storage_driver_manager().await.unwrap();
        let result = save_delay_message(
            &storage,
            "missing-topic",
            AdapterWriteRecord::new("source", b"payload".to_vec()),
            1,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn saving_delay_index_requires_registered_topic() {
        let storage = test_build_storage_driver_manager().await.unwrap();
        let result = save_delay_index_info(&storage, &delay_info("missing-index-topic"), 1).await;
        assert!(result.is_err());
    }
}
