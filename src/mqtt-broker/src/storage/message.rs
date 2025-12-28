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
use std::collections::HashMap;
use storage_adapter::storage::ArcStorageAdapter;

#[derive(Clone)]
pub struct MessageStorage {
    pub storage_adapter: ArcStorageAdapter,
}

impl MessageStorage {
    pub fn new(storage_adapter: ArcStorageAdapter) -> Self {
        MessageStorage { storage_adapter }
    }

    pub async fn append_topic_message(
        &self,
        topic_name: &str,
        record: Vec<AdapterWriteRecord>,
    ) -> Result<Vec<u64>, CommonError> {
        let shard_name = topic_name;
        let results = self
            .storage_adapter
            .batch_write(shard_name, &record)
            .await?;
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

        let engine_records = self
            .storage_adapter
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
        shard_name: &str,
    ) -> Result<u64, CommonError> {
        for row in self
            .storage_adapter
            .get_offset_by_group(group_id)
            .await?
            .iter()
        {
            if *shard_name == row.shard_name {
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

        self.storage_adapter
            .commit_offset(group_id, &offset_data)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::storage::memory::StorageDriverMemoryConfig;
    use metadata_struct::storage::{
        adapter_offset::AdapterShardInfo, adapter_record::AdapterWriteRecord,
    };
    use std::sync::Arc;
    use storage_adapter::memory::MemoryStorageAdapter;
    use storage_engine::memory::engine::MemoryStorageEngine;

    async fn create_test_storage() -> MessageStorage {
        let memory_storage_engine = Arc::new(MemoryStorageEngine::new(
            StorageDriverMemoryConfig::default(),
        ));
        let storage_adapter = Arc::new(MemoryStorageAdapter::new(memory_storage_engine));
        MessageStorage::new(storage_adapter)
    }

    #[tokio::test]
    async fn test_message_read_write_with_metadata() {
        let storage = create_test_storage().await;
        let shard_name = "topic1";
        storage
            .storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: shard_name.to_string(),
                replica_num: 1,
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

        let offsets = storage
            .append_topic_message(shard_name, records)
            .await
            .unwrap();
        assert_eq!(offsets.len(), 10);

        // Test read with offset and limit
        let msgs = storage
            .read_topic_message(shard_name, offsets[5], 3)
            .await
            .unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(String::from_utf8_lossy(&msgs[0].data), "Message 5");
        assert_eq!(msgs[0].key(), Some("key5"));
        assert_eq!(msgs[0].tags(), &["tag5"]);
    }

    #[tokio::test]
    async fn test_group_offset_isolation() {
        let storage = create_test_storage().await;
        storage
            .storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: "t1".to_string(),
                replica_num: 1,
            })
            .await
            .unwrap();

        storage
            .storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: "t2".to_string(),
                replica_num: 1,
            })
            .await
            .unwrap();

        // Initial offset is 0
        assert_eq!(storage.get_group_offset("g1", "t1").await.unwrap(), 0);

        // Multiple groups with multiple topics
        storage.commit_group_offset("g1", "t1", 10).await.unwrap();
        storage.commit_group_offset("g1", "t2", 20).await.unwrap();
        storage.commit_group_offset("g2", "t1", 30).await.unwrap();

        // Verify isolation
        assert_eq!(storage.get_group_offset("g1", "t1").await.unwrap(), 10);
        assert_eq!(storage.get_group_offset("g1", "t2").await.unwrap(), 20);
        assert_eq!(storage.get_group_offset("g2", "t1").await.unwrap(), 30);
    }

    #[tokio::test]
    async fn test_consumer_flow() {
        let storage = create_test_storage().await;
        let shard_name = "topic1";
        storage
            .storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: shard_name.to_string(),
                replica_num: 1,
            })
            .await
            .unwrap();

        // Append messages
        let records: Vec<AdapterWriteRecord> = (0..10)
            .map(|i| AdapterWriteRecord::from_string(format!("Msg{}", i)))
            .collect();
        let offsets = storage
            .append_topic_message(shard_name, records)
            .await
            .unwrap();

        // First consume: read and commit
        let offset = storage.get_group_offset("group", shard_name).await.unwrap();
        let batch1 = storage
            .read_topic_message(shard_name, offset, 5)
            .await
            .unwrap();
        assert_eq!(batch1.len(), 5);
        storage
            .commit_group_offset("group", shard_name, offsets[5])
            .await
            .unwrap();

        // Second consume: continue from last committed
        let offset = storage.get_group_offset("group", shard_name).await.unwrap();
        let batch2 = storage
            .read_topic_message(shard_name, offset, 5)
            .await
            .unwrap();
        assert_eq!(batch2.len(), 5);
        assert_eq!(String::from_utf8_lossy(&batch2[0].data), "Msg5");
    }
}
