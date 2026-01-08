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

use std::sync::Arc;

use crate::{
    commitlog::{
        offset::{get_latest_offset, save_latest_offset},
        rocksdb::engine::{IndexInfo, RocksDBStorageEngine},
    },
    core::error::StorageEngineError,
};
use common_base::{
    tools::now_second,
    utils::serialize::{self, serialize},
};
use metadata_struct::storage::{
    adapter_read_config::AdapterWriteRespRow, adapter_record::AdapterWriteRecord,
    convert::convert_adapter_record_to_engine,
};
use rocksdb::WriteBatch;
use rocksdb_engine::keys::storage::{
    key_index_key, shard_record_key, tag_index_key, timestamp_index_key,
};

impl RocksDBStorageEngine {
    pub async fn write(
        &self,
        shard: &str,
        message: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, StorageEngineError> {
        let results = self
            .batch_write_internal(shard, std::slice::from_ref(message))
            .await?;

        if results.is_empty() {
            return Err(StorageEngineError::CommonErrorStr(
                "Write operation returned empty result".to_string(),
            ));
        }

        results.first().cloned().ok_or_else(|| {
            StorageEngineError::CommonErrorStr("Empty offset result from write".to_string())
        })
    }

    pub async fn batch_write(
        &self,
        shard: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        self.batch_write_internal(shard, messages).await
    }

    async fn batch_write_internal(
        &self,
        shard_name: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let lock = self
            .shard_write_locks
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();

        let _guard = lock.lock().await;

        let cf = self.get_cf()?;
        let mut offset = get_latest_offset(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            shard_name,
        )?;

        let mut results = Vec::with_capacity(messages.len());
        let mut batch = WriteBatch::default();

        for msg in messages {
            results.push(AdapterWriteRespRow {
                pkid: msg.pkid,
                offset,
                ..Default::default()
            });

            // Convert StorageAdapterRecord to StorageEngineRecord
            let engine_record = convert_adapter_record_to_engine(msg.clone(), shard_name, offset);

            // save message (now storing StorageEngineRecord)
            let shard_record_key = shard_record_key(shard_name, offset);
            let serialized_msg = serialize::serialize(&engine_record)?;
            batch.put_cf(&cf, shard_record_key.as_bytes(), &serialized_msg);

            // save index
            let offset_info = IndexInfo {
                shard_name: shard_name.to_string(),
                offset,
                create_time: now_second(),
            };
            let offset_info_data = serialize(&offset_info)?;

            // key index
            if let Some(key) = &msg.key {
                let key_index_key = key_index_key(shard_name, key);
                batch.put_cf(&cf, key_index_key.as_bytes(), offset_info_data.clone());
            }

            // tag index
            if let Some(tags) = &msg.tags {
                for tag in tags.iter() {
                    let tag_index_key = tag_index_key(shard_name, tag, offset);
                    batch.put_cf(&cf, tag_index_key.as_bytes(), offset_info_data.clone());
                }
            }

            // timestamp index
            if msg.timestamp > 0 && offset % 5000 == 0 {
                let timestamp_index_key = timestamp_index_key(shard_name, msg.timestamp, offset);
                batch.put_cf(
                    &cf,
                    timestamp_index_key.as_bytes(),
                    offset_info_data.clone(),
                );
            }

            // offset incr
            offset += 1;
        }
        self.rocksdb_engine_handler.write_batch(batch)?;
        save_latest_offset(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            shard_name,
            offset,
        )?;
        Ok(results)
    }

    pub async fn delete_by_key(&self, shard: &str, key: &str) -> Result<(), StorageEngineError> {
        let index = if let Some(index) = self.get_offset_by_key(shard, key).await? {
            index
        } else {
            return Ok(());
        };

        self.delete_by_offset(shard, index.offset).await
    }

    pub async fn delete_by_offset(
        &self,
        shard: &str,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let cf = self.get_cf()?;
        let record_key = shard_record_key(shard, offset);

        let record = match self
            .rocksdb_engine_handler
            .read::<metadata_struct::storage::storage_record::StorageRecord>(
            cf.clone(),
            &record_key,
        )? {
            Some(r) => r,
            None => return Ok(()),
        };

        let mut batch = WriteBatch::default();

        batch.delete_cf(&cf, record_key.as_bytes());

        if let Some(key) = &record.metadata.key {
            let key_index_key = key_index_key(shard, key);
            batch.delete_cf(&cf, key_index_key.as_bytes());
        }

        if let Some(tags) = &record.metadata.tags {
            for tag in tags.iter() {
                let tag_index_key = tag_index_key(shard, tag, offset);
                batch.delete_cf(&cf, tag_index_key.as_bytes());
            }
        }

        if record.metadata.create_t > 0 && offset.is_multiple_of(5000) {
            let timestamp_index_key = timestamp_index_key(shard, record.metadata.create_t, offset);
            batch.delete_cf(&cf, timestamp_index_key.as_bytes());
        }

        self.rocksdb_engine_handler.write_batch(batch)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_tool::test_build_rocksdb_engine;
    use bytes::Bytes;
    use common_base::tools::unique_id;
    use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
    use metadata_struct::storage::adapter_record::AdapterWriteRecord;

    #[tokio::test]
    async fn test_write_and_delete() {
        let engine = test_build_rocksdb_engine();
        let shard_name = unique_id();

        let messages: Vec<AdapterWriteRecord> = (0..5)
            .map(|i| AdapterWriteRecord {
                pkid: i,
                key: Some(format!("key{}", i)),
                tags: Some(vec![format!("tag{}", i)]),
                data: Bytes::from(format!("data{}", i)),
                timestamp: 1000 + i * 100,
                ..Default::default()
            })
            .collect();

        engine.batch_write(&shard_name, &messages).await.unwrap();

        engine.delete_by_key(&shard_name, "key2").await.unwrap();

        let read_config = AdapterReadConfig {
            max_record_num: 10,
            max_size: 1024 * 1024,
        };

        let key_records = engine.read_by_key(&shard_name, "key2").await.unwrap();
        assert_eq!(key_records.len(), 0);

        let tag_records = engine
            .read_by_tag(&shard_name, "tag2", None, &read_config)
            .await
            .unwrap();
        assert_eq!(tag_records.len(), 0);
    }
}
