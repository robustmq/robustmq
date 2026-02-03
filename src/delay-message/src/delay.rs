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
use crate::manager::DelayMessageManager;
use broker_core::cache::BrokerCacheManager;
use bytes::Bytes;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_base::utils::serialize::serialize;
use common_base::uuid::unique_id;
use common_config::storage::StorageType;
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use storage_adapter::topic::create_topic_full;
use tracing::debug;

pub const DELAY_QUEUE_MESSAGE_TOPIC: &str = "$delay-queue-message";
pub const DELAY_QUEUE_INDEX_TOPIC: &str = "$delay-queue-index";

pub(crate) async fn save_delay_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_message_id: &str,
    mut data: AdapterWriteRecord,
) -> Result<u64, CommonError> {
    data.key = Some(delay_message_id.to_string());
    let result = storage_driver_manager
        .write(DELAY_QUEUE_MESSAGE_TOPIC, &[data])
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
        .delete_by_key(DELAY_QUEUE_MESSAGE_TOPIC, unique_id)
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
    let data = AdapterWriteRecord {
        key: Some(delay_info.unique_id.clone()),
        data: Bytes::copy_from_slice(&data),
        timestamp: now_second(),
        ..Default::default()
    };
    let result = storage_driver_manager
        .write(DELAY_QUEUE_INDEX_TOPIC, &[data])
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
        .delete_by_key(DELAY_QUEUE_INDEX_TOPIC, &delay_info.unique_id)
        .await?;
    debug!(
        "Deleted delay index info: unique_id={}",
        delay_info.unique_id
    );
    Ok(())
}

pub(crate) async fn init_inner_topic(
    delay_message_manager: &Arc<DelayMessageManager>,
    broker_cache: &Arc<BrokerCacheManager>,
) -> Result<(), CommonError> {
    for topic_name in [
        DELAY_QUEUE_MESSAGE_TOPIC.to_string(),
        DELAY_QUEUE_INDEX_TOPIC.to_string(),
    ] {
        let uid = unique_id();
        let topic = Topic {
            topic_id: uid.clone(),
            topic_name: topic_name.to_string(),
            storage_type: StorageType::EngineRocksDB,
            partition: 1,
            replication: 1,
            storage_name_list: Topic::create_partition_name(&uid, 1),
            create_time: now_second(),
        };
        let shard_config = build_delay_message_shard_config(&StorageType::EngineRocksDB)?;
        create_topic_full(
            broker_cache,
            &delay_message_manager.storage_driver_manager,
            &delay_message_manager.client_pool,
            &topic,
            &shard_config,
        )
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    // use common_base::{tools::unique_id, utils::serialize};
    // use common_config::storage::StorageType;
    // use metadata_struct::{
    //     delay_info::DelayMessageIndexInfo,
    //     storage::{adapter_offset::AdapterShardInfo, adapter_record::AdapterWriteRecord},
    // };
    // use storage_adapter::storage::test_build_memory_storage_driver;

    // use crate::{
    //     delay::{
    //         delete_delay_index_info, delete_delay_message, get_delay_message_topic_name,
    //         init_inner_topic, save_delay_index_info, save_delay_message,
    //         DELAY_QUEUE_INFO_SHARD_NAME,
    //     },
    //     pop::read_offset_data,
    // };

    // #[tokio::test]
    // async fn shard_name_format_test() {
    //     assert_eq!(
    //         get_delay_message_topic_name(0),
    //         "$delay-queue-message-shard-0"
    //     );
    // }

    // #[tokio::test]
    // async fn shard_init_test() {
    //     let adapter = test_build_memory_storage_driver();
    //     init_inner_topic(&adapter, &StorageType::EngineMemory, 2)
    //         .await
    //         .unwrap();

    //     let all_shards = adapter.list_shard(None).await.unwrap();
    //     assert_eq!(all_shards.len(), 3);

    //     let shard_names: Vec<_> = all_shards.iter().map(|s| s.shard_name.as_str()).collect();
    //     assert!(shard_names.contains(&get_delay_message_topic_name(0).as_str()));
    //     assert!(shard_names.contains(&get_delay_message_topic_name(1).as_str()));
    //     assert!(shard_names.contains(&DELAY_QUEUE_INFO_SHARD_NAME));
    // }

    // #[tokio::test]
    // async fn message_crud_test() {
    //     let adapter = test_build_memory_storage_driver();
    //     let shard_name = unique_id();
    //     adapter
    //         .create_shard(&AdapterShardInfo {
    //             shard_name: shard_name.clone(),
    //             ..Default::default()
    //         })
    //         .await
    //         .unwrap();

    //     let data = AdapterWriteRecord::from_string("test_data".to_string());
    //     let offset = save_delay_message(&adapter, &shard_name, data)
    //         .await
    //         .unwrap();

    //     let record = read_offset_data(&adapter, &shard_name, offset)
    //         .await
    //         .unwrap()
    //         .unwrap();
    //     assert_eq!(record.pkid, offset);
    //     assert_eq!(
    //         String::from_utf8(record.data.to_vec()).unwrap(),
    //         "test_data"
    //     );

    //     delete_delay_message(
    //         &adapter,
    //         &DelayMessageIndexInfo {
    //             unique_id: unique_id(),
    //             delay_shard_name: shard_name.clone(),
    //             target_topic_name: "".to_string(),
    //             offset,
    //             delay_timestamp: 0,
    //             shard_no: 0,
    //         },
    //     )
    //     .await
    //     .unwrap();

    //     let result = read_offset_data(&adapter, &shard_name, offset).await;
    //     assert!(result.is_ok());
    //     assert!(result.unwrap().is_none());
    // }

    // #[tokio::test]
    // async fn index_info_crud_test() {
    //     let adapter = test_build_memory_storage_driver();
    //     init_inner_topic(&adapter, &StorageType::EngineMemory, 1)
    //         .await
    //         .unwrap();

    //     let delay_info = DelayMessageIndexInfo {
    //         unique_id: unique_id(),
    //         delay_shard_name: "test_delay_shard".to_string(),
    //         target_topic_name: "test_target".to_string(),
    //         offset: 100,
    //         delay_timestamp: 1000,
    //         shard_no: 0,
    //     };

    //     save_delay_index_info(&adapter, &delay_info).await.unwrap();

    //     let record = read_offset_data(&adapter, DELAY_QUEUE_INFO_SHARD_NAME, 0)
    //         .await
    //         .unwrap()
    //         .unwrap();
    //     let saved_info = serialize::deserialize::<DelayMessageIndexInfo>(&record.data).unwrap();
    //     assert_eq!(saved_info.unique_id, delay_info.unique_id);
    //     assert_eq!(saved_info.offset, 100);
    //     assert_eq!(saved_info.delay_timestamp, 1000);

    //     delete_delay_index_info(&adapter, &delay_info)
    //         .await
    //         .unwrap();

    //     let key_result = adapter
    //         .read_by_key(DELAY_QUEUE_INFO_SHARD_NAME, &delay_info.unique_id)
    //         .await
    //         .unwrap();
    //     assert_eq!(key_result.len(), 0);
    // }
}
