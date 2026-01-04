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

use common_base::error::common::CommonError;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::convert::convert_engine_record_to_adapter;
use std::{collections::HashMap, sync::Arc};
use storage_adapter::driver::StorageDriverManager;

use crate::storage::driver::get_driver_by_mqtt_topic_name;

#[derive(Clone)]
pub struct MessageStorage {
    pub storage_driver_manager: Arc<StorageDriverManager>,
}

impl MessageStorage {
    pub fn new(storage_driver_manager: Arc<StorageDriverManager>) -> Self {
        MessageStorage {
            storage_driver_manager,
        }
    }

    pub async fn append_topic_message(
        &self,
        topic_name: &str,
        record: Vec<AdapterWriteRecord>,
    ) -> Result<Vec<u64>, CommonError> {
        let shard_name = topic_name;
        let driver = get_driver_by_mqtt_topic_name(&self.storage_driver_manager, topic_name)?;
        let results = driver.batch_write(shard_name, &record).await?;
        let mut offsets = Vec::new();
        for row in results {
            if row.is_error() {
                return Err(CommonError::CommonError(row.error_info()));
            }
            offsets.push(row.offset);
        }
        Ok(offsets)
    }

    pub async fn read_topic_message(
        &self,
        topic_name: &str,
        offset: u64,
        record_num: u64,
    ) -> Result<Vec<AdapterWriteRecord>, CommonError> {
        let shard_name = topic_name;

        let mut read_config = AdapterReadConfig::new();
        read_config.max_record_num = record_num;

        let driver = get_driver_by_mqtt_topic_name(&self.storage_driver_manager, topic_name)?;
        let engine_records = driver
            .read_by_offset(shard_name, offset, &read_config)
            .await?;

        let mut adapter_records = Vec::with_capacity(engine_records.len());
        for raw in engine_records {
            let calculated_crc = common_base::utils::crc::calc_crc32(&raw.data);
            if raw.metadata.crc_num != calculated_crc {
                return Err(CommonError::CrcCheckByMessage);
            }
            adapter_records.push(convert_engine_record_to_adapter(raw));
        }
        Ok(adapter_records)
    }

    pub async fn get_group_offset(
        &self,
        group_id: &str,
        topic_name: &str,
    ) -> Result<u64, CommonError> {
        let driver = get_driver_by_mqtt_topic_name(&self.storage_driver_manager, topic_name)?;
        for row in driver.get_offset_by_group(group_id).await?.iter() {
            if *topic_name == row.shard_name {
                return Ok(row.offset);
            }
        }
        Ok(0)
    }

    pub async fn commit_group_offset(
        &self,
        group_id: &str,
        topic_name: &str,
        offset: u64,
    ) -> Result<(), CommonError> {
        let shard_name = topic_name;
        let mut offset_data = HashMap::new();
        offset_data.insert(shard_name.to_owned(), offset);

        let driver = get_driver_by_mqtt_topic_name(&self.storage_driver_manager, topic_name)?;
        driver.commit_offset(group_id, &offset_data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::unique_id;
    use metadata_struct::storage::{
        adapter_offset::AdapterShardInfo, adapter_record::AdapterWriteRecord,
        shard::EngineShardConfig,
    };
    use storage_adapter::storage::{test_build_storage_driver_manager, StorageAdapter};
    async fn create_test_storage() -> MessageStorage {
        let memory_storage_engine = test_build_storage_driver_manager().await.unwrap();
        MessageStorage::new(memory_storage_engine)
    }

    #[tokio::test]
    async fn test_message_read_write_with_metadata() {
        let topic_name = unique_id();
        let message_storage = create_test_storage().await;

        // create shard
        let storage_adapter: Arc<dyn StorageAdapter + Send + Sync> =
            get_driver_by_mqtt_topic_name(&message_storage.storage_driver_manager, &topic_name)
                .unwrap();

        storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: topic_name.to_string(),
                config: EngineShardConfig::default(),
            })
            .await
            .unwrap();

        // Test basic append and read
        let records: Vec<AdapterWriteRecord> = (0..10)
            .map(|i| {
                AdapterWriteRecord::from_string(format!("Message {}", i))
                    .with_key(format!("key{}", i))
                    .with_tags(vec![format!("tag{}", i)])
            })
            .collect();

        let offsets = message_storage
            .append_topic_message(&topic_name, records)
            .await
            .unwrap();
        assert_eq!(offsets.len(), 10);

        // Test read with offset and limit
        let msgs = message_storage
            .read_topic_message(&topic_name, offsets[5], 3)
            .await
            .unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(String::from_utf8_lossy(&msgs[0].data), "Message 5");
        assert_eq!(msgs[0].key(), Some("key5"));
        assert_eq!(msgs[0].tags(), &["tag5"]);
    }

    #[tokio::test]
    async fn test_group_offset_isolation() {
        let topic_name = unique_id();
        let message_storage = create_test_storage().await;
        let storage_adapter: Arc<dyn StorageAdapter + Send + Sync> =
            get_driver_by_mqtt_topic_name(&message_storage.storage_driver_manager, &topic_name)
                .unwrap();

        storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: "t1".to_string(),
                config: EngineShardConfig::default(),
            })
            .await
            .unwrap();

        storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: "t2".to_string(),
                config: EngineShardConfig::default(),
            })
            .await
            .unwrap();

        // Initial offset is 0
        assert_eq!(
            message_storage.get_group_offset("g1", "t1").await.unwrap(),
            0
        );

        // Multiple groups with multiple topics
        message_storage
            .commit_group_offset("g1", "t1", 10)
            .await
            .unwrap();
        message_storage
            .commit_group_offset("g1", "t2", 20)
            .await
            .unwrap();
        message_storage
            .commit_group_offset("g2", "t1", 30)
            .await
            .unwrap();

        // Verify isolation
        assert_eq!(
            message_storage.get_group_offset("g1", "t1").await.unwrap(),
            10
        );
        assert_eq!(
            message_storage.get_group_offset("g1", "t2").await.unwrap(),
            20
        );
        assert_eq!(
            message_storage.get_group_offset("g2", "t1").await.unwrap(),
            30
        );
    }

    #[tokio::test]
    async fn test_consumer_flow() {
        let topic_name = unique_id();
        let message_storage = create_test_storage().await;
        let storage_adapter: Arc<dyn StorageAdapter + Send + Sync> =
            get_driver_by_mqtt_topic_name(&message_storage.storage_driver_manager, &topic_name)
                .unwrap();
        storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: topic_name.to_string(),
                config: EngineShardConfig::default(),
            })
            .await
            .unwrap();

        // Append messages
        let records: Vec<AdapterWriteRecord> = (0..10)
            .map(|i| AdapterWriteRecord::from_string(format!("Msg{}", i)))
            .collect();
        let offsets = message_storage
            .append_topic_message(&topic_name, records)
            .await
            .unwrap();

        // First consume: read and commit
        let offset = message_storage
            .get_group_offset("group", &topic_name)
            .await
            .unwrap();
        let batch1 = message_storage
            .read_topic_message(&topic_name, offset, 5)
            .await
            .unwrap();
        assert_eq!(batch1.len(), 5);
        message_storage
            .commit_group_offset("group", &topic_name, offsets[5])
            .await
            .unwrap();

        // Second consume: continue from last committed
        let offset = message_storage
            .get_group_offset("group", &topic_name)
            .await
            .unwrap();
        let batch2 = message_storage
            .read_topic_message(&topic_name, offset, 5)
            .await
            .unwrap();
        assert_eq!(batch2.len(), 5);
        assert_eq!(String::from_utf8_lossy(&batch2[0].data), "Msg5");
    }
}
