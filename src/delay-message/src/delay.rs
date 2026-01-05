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

use crate::driver::build_delay_message_shard_config;
use bytes::Bytes;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_base::utils::serialize::serialize;
use common_config::storage::StorageAdapterType;
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::{adapter_offset::AdapterShardInfo, shard::EngineShardConfig};
use std::collections::HashSet;
use storage_adapter::driver::ArcStorageAdapter;
use tracing::{debug, info};

const DELAY_MESSAGE_SHARD_NAME_PREFIX: &str = "$delay-queue-message-shard-";
pub const DELAY_QUEUE_INFO_SHARD_NAME: &str = "$delay-queue-index-info-shard";

pub(crate) async fn save_delay_message(
    message_storage_adapter: &ArcStorageAdapter,
    shard_name: &str,
    data: AdapterWriteRecord,
) -> Result<u64, CommonError> {
    let resp = message_storage_adapter.write(shard_name, &data).await?;
    if resp.is_error() {
        return Err(CommonError::CommonError(resp.error_info()));
    }

    debug!(
        "Delay message persisted to shard {} at offset {}",
        shard_name, resp.offset
    );

    Ok(resp.offset)
}

pub(crate) async fn delete_delay_message(
    message_storage_adapter: &ArcStorageAdapter,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    message_storage_adapter
        .delete_by_offset(&delay_info.delay_shard_name, delay_info.offset)
        .await?;
    debug!(
        "Deleted delay message: shard={}, offset={}",
        delay_info.delay_shard_name, delay_info.offset
    );
    Ok(())
}

pub async fn save_delay_index_info(
    message_storage_adapter: &ArcStorageAdapter,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    let data = serialize(&delay_info)?;
    let data = AdapterWriteRecord {
        key: Some(delay_info.unique_id.clone()),
        data: Bytes::copy_from_slice(&data),
        timestamp: now_second(),
        ..Default::default()
    };
    let resp = message_storage_adapter
        .write(DELAY_QUEUE_INFO_SHARD_NAME, &data)
        .await?;
    if resp.is_error() {
        return Err(CommonError::CommonError(resp.error_info()));
    }
    Ok(())
}

pub(crate) async fn delete_delay_index_info(
    message_storage_adapter: &ArcStorageAdapter,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    message_storage_adapter
        .delete_by_key(DELAY_QUEUE_INFO_SHARD_NAME, &delay_info.unique_id)
        .await?;
    debug!(
        "Deleted delay index info: unique_id={}",
        delay_info.unique_id
    );
    Ok(())
}

pub(crate) async fn init_delay_message_shard(
    message_storage_adapter: &ArcStorageAdapter,
    engine_storage_type: &StorageAdapterType,
    shard_num: u64,
) -> Result<(), CommonError> {
    let all_shards = message_storage_adapter.list_shard(None).await?;
    let existing_names: HashSet<_> = all_shards.iter().map(|s| s.shard_name.as_str()).collect();

    let mut created_count = 0;
    for i in 0..shard_num {
        let shard_name = get_delay_message_shard_name(i);
        if !existing_names.contains(shard_name.as_str()) {
            let shard = AdapterShardInfo {
                shard_name: shard_name.clone(),
                config: build_delay_message_shard_config(engine_storage_type)?,
            };
            message_storage_adapter.create_shard(&shard).await?;
            info!("Created delay message shard: {}", shard_name);
            created_count += 1;
        }
    }

    if !existing_names.contains(DELAY_QUEUE_INFO_SHARD_NAME) {
        let shard = AdapterShardInfo {
            shard_name: DELAY_QUEUE_INFO_SHARD_NAME.to_string(),
            config: EngineShardConfig::default(),
        };
        message_storage_adapter.create_shard(&shard).await?;
        info!(
            "Created delay message shard: {}",
            DELAY_QUEUE_INFO_SHARD_NAME
        );
        created_count += 1;
    }

    info!(
        "Delay message shards initialized: {} total, {} newly created",
        shard_num + 1,
        created_count
    );

    Ok(())
}

pub(crate) fn get_delay_message_shard_name(no: u64) -> String {
    format!("{DELAY_MESSAGE_SHARD_NAME_PREFIX}{no}")
}

#[cfg(test)]
mod test {
    use common_base::{tools::unique_id, utils::serialize};
    use common_config::storage::StorageAdapterType;
    use metadata_struct::{
        delay_info::DelayMessageIndexInfo,
        storage::{adapter_offset::AdapterShardInfo, adapter_record::AdapterWriteRecord},
    };
    use storage_adapter::storage::test_build_memory_storage_driver;

    use crate::{
        delay::{
            delete_delay_index_info, delete_delay_message, get_delay_message_shard_name,
            init_delay_message_shard, save_delay_index_info, save_delay_message,
            DELAY_QUEUE_INFO_SHARD_NAME,
        },
        pop::read_offset_data,
    };

    #[tokio::test]
    async fn shard_name_format_test() {
        assert_eq!(
            get_delay_message_shard_name(0),
            "$delay-queue-message-shard-0"
        );
    }

    #[tokio::test]
    async fn shard_init_test() {
        let adapter = test_build_memory_storage_driver();
        init_delay_message_shard(&adapter, &StorageAdapterType::Memory, 2)
            .await
            .unwrap();

        let all_shards = adapter.list_shard(None).await.unwrap();
        assert_eq!(all_shards.len(), 3);

        let shard_names: Vec<_> = all_shards.iter().map(|s| s.shard_name.as_str()).collect();
        assert!(shard_names.contains(&get_delay_message_shard_name(0).as_str()));
        assert!(shard_names.contains(&get_delay_message_shard_name(1).as_str()));
        assert!(shard_names.contains(&DELAY_QUEUE_INFO_SHARD_NAME));
    }

    #[tokio::test]
    async fn message_crud_test() {
        let adapter = test_build_memory_storage_driver();
        let shard_name = unique_id();
        adapter
            .create_shard(&AdapterShardInfo {
                shard_name: shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let data = AdapterWriteRecord::from_string("test_data".to_string());
        let offset = save_delay_message(&adapter, &shard_name, data)
            .await
            .unwrap();

        let record = read_offset_data(&adapter, &shard_name, offset)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(record.pkid, offset);
        assert_eq!(
            String::from_utf8(record.data.to_vec()).unwrap(),
            "test_data"
        );

        delete_delay_message(
            &adapter,
            &DelayMessageIndexInfo {
                unique_id: unique_id(),
                delay_shard_name: shard_name.clone(),
                target_shard_name: "".to_string(),
                offset,
                delay_timestamp: 0,
                shard_no: 0,
            },
        )
        .await
        .unwrap();

        let result = read_offset_data(&adapter, &shard_name, offset).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn index_info_crud_test() {
        let adapter = test_build_memory_storage_driver();
        init_delay_message_shard(&adapter, &StorageAdapterType::Memory, 1)
            .await
            .unwrap();

        let delay_info = DelayMessageIndexInfo {
            unique_id: unique_id(),
            delay_shard_name: "test_delay_shard".to_string(),
            target_shard_name: "test_target".to_string(),
            offset: 100,
            delay_timestamp: 1000,
            shard_no: 0,
        };

        save_delay_index_info(&adapter, &delay_info).await.unwrap();

        let record = read_offset_data(&adapter, DELAY_QUEUE_INFO_SHARD_NAME, 0)
            .await
            .unwrap()
            .unwrap();
        let saved_info = serialize::deserialize::<DelayMessageIndexInfo>(&record.data).unwrap();
        assert_eq!(saved_info.unique_id, delay_info.unique_id);
        assert_eq!(saved_info.offset, 100);
        assert_eq!(saved_info.delay_timestamp, 1000);

        delete_delay_index_info(&adapter, &delay_info)
            .await
            .unwrap();

        let key_result = adapter
            .read_by_key(DELAY_QUEUE_INFO_SHARD_NAME, &delay_info.unique_id)
            .await
            .unwrap();
        assert_eq!(key_result.len(), 0);
    }
}
