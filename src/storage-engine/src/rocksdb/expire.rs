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

use crate::{core::error::StorageEngineError, rocksdb::engine::RocksDBStorageEngine};
use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::{loop_select_ticket, now_second},
    utils::serialize::{self},
};
use futures::stream::StreamExt;
use metadata_struct::storage::{
    adapter_offset::AdapterOffsetStrategy, shard::EngineShard, storage_record::StorageRecord,
};
use rocksdb::WriteBatch;
use rocksdb_engine::keys::storage::{
    key_index_key, shard_info_key_prefix, shard_record_key, tag_index_key, timestamp_index_key,
};
use tokio::sync::broadcast;

impl RocksDBStorageEngine {
    pub async fn start_expire_thread(&self, stop_sx: &broadcast::Sender<bool>) {
        let ac_fn = async || -> ResultCommonError {
            self.scan_and_delete_expire_data()
                .await
                .map_err(|e| CommonError::CommonError(e.to_string()))?;
            Ok(())
        };
        loop_select_ticket(ac_fn, 600000, stop_sx).await;
    }

    async fn scan_and_delete_expire_data(&self) -> Result<(), StorageEngineError> {
        let shard_infos = {
            let cf = self.get_cf()?;
            let raw_shard_info = self
                .rocksdb_engine_handler
                .read_prefix(cf.clone(), &shard_info_key_prefix())?;

            let mut shard_infos = Vec::new();
            for (_, v) in raw_shard_info {
                let info = serialize::deserialize::<EngineShard>(v.as_slice())?;
                shard_infos.push(info);
            }
            shard_infos
        };

        if shard_infos.is_empty() {
            return Ok(());
        }

        const MAX_CONCURRENT_TASKS: usize = 10;

        let mut stream = futures::stream::iter(shard_infos)
            .map(|shard_info| self.scan_and_delete_data_by_shard(shard_info))
            .buffer_unordered(MAX_CONCURRENT_TASKS);

        while let Some(result) = stream.next().await {
            result?;
        }

        Ok(())
    }

    async fn scan_and_delete_data_by_shard(
        &self,
        shard: EngineShard,
    ) -> Result<(), StorageEngineError> {
        let earliest_timestamp = now_second() - shard.config.retention_sec;
        let earliest_offset = self.get_earliest_offset(&shard.shard_name)?;
        let new_earliest_offset = self
            .get_offset_by_timestamp(
                &shard.shard_name,
                earliest_timestamp,
                AdapterOffsetStrategy::Earliest,
            )
            .await?;

        if new_earliest_offset <= earliest_offset {
            return Ok(());
        }

        let cf = self.get_cf()?;
        let mut batch = WriteBatch::default();

        const BATCH_SIZE: u64 = 1000;
        let mut current_offset = earliest_offset;

        while current_offset < new_earliest_offset {
            let batch_end = (current_offset + BATCH_SIZE).min(new_earliest_offset);
            let keys: Vec<String> = (current_offset..batch_end)
                .map(|off| shard_record_key(&shard.shard_name, off))
                .collect();

            let records = self
                .rocksdb_engine_handler
                .multi_get::<StorageRecord>(cf.clone(), &keys)?;

            for (i, record_opt) in records.into_iter().enumerate() {
                if let Some(record) = record_opt {
                    let offset = current_offset + i as u64;
                    let record_key = &keys[i];

                    batch.delete_cf(&cf, record_key.as_bytes());

                    if let Some(key) = &record.metadata.key {
                        let key_idx = key_index_key(&shard.shard_name, key);
                        batch.delete_cf(&cf, key_idx.as_bytes());
                    }

                    if let Some(tags) = &record.metadata.tags {
                        for tag in tags.iter() {
                            let tag_idx = tag_index_key(&shard.shard_name, tag, offset);
                            batch.delete_cf(&cf, tag_idx.as_bytes());
                        }
                    }

                    if offset.is_multiple_of(5000) && record.metadata.create_t > 0 {
                        let ts_idx = timestamp_index_key(
                            &shard.shard_name,
                            record.metadata.create_t,
                            offset,
                        );
                        batch.delete_cf(&cf, ts_idx.as_bytes());
                    }
                }
            }

            current_offset = batch_end;
        }

        self.rocksdb_engine_handler.write_batch(batch)?;
        self.save_earliest_offset(&shard.shard_name, new_earliest_offset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{shard::StorageEngineRunType, test_tool::test_build_engine};
    use bytes::Bytes;
    use common_base::tools::{now_second, unique_id};
    use common_config::storage::StorageType;
    use metadata_struct::storage::{
        adapter_offset::AdapterShardInfo,
        adapter_read_config::AdapterReadConfig,
        adapter_record::AdapterWriteRecord,
        shard::{EngineShardConfig, EngineStorageType},
    };

    #[tokio::test]
    async fn test_scan_and_delete_expire_data() {
        let engine = test_build_engine(StorageEngineRunType::Standalone);
        let shard_name = unique_id();
        let now = now_second();

        let shard_info = AdapterShardInfo {
            shard_name: shard_name.clone(),
            config: EngineShardConfig {
                retention_sec: 10,
                storage_adapter_type: StorageType::EngineSegment,
                engine_storage_type: Some(EngineStorageType::EngineRocksDB),
                ..Default::default()
            },
        };
        engine.create_shard(&shard_info).await.unwrap();

        let mut expired_records = Vec::new();
        for i in 0..5 {
            let record = AdapterWriteRecord {
                data: Bytes::from(format!("expired{}", i)),
                key: Some(format!("exp_key{}", i)),
                tags: Some(vec!["exp_tag".to_string()]),
                timestamp: now - 20,
                ..Default::default()
            };
            expired_records.push(record);
        }

        let mut valid_records = Vec::new();
        for i in 5..10 {
            let record = AdapterWriteRecord {
                data: Bytes::from(format!("valid{}", i)),
                key: Some(format!("val_key{}", i)),
                tags: Some(vec!["val_tag".to_string()]),
                timestamp: now - 5,
                ..Default::default()
            };
            valid_records.push(record);
        }

        let mut all_records = expired_records.clone();
        all_records.extend(valid_records.clone());
        engine.batch_write(&shard_name, &all_records).await.unwrap();

        engine.scan_and_delete_expire_data().await.unwrap();

        let earliest_after = engine.get_earliest_offset(&shard_name).unwrap();

        let read_config = AdapterReadConfig {
            max_record_num: 20,
            max_size: 1024 * 1024,
        };
        let records = engine
            .read_by_offset(&shard_name, earliest_after, &read_config)
            .await
            .unwrap();
        assert_eq!(records.len(), 5);
        assert_eq!(records[0].metadata.offset, 5);

        let expired_key_records = engine.read_by_key(&shard_name, "exp_key0").await.unwrap();
        assert_eq!(expired_key_records.len(), 0);

        let valid_key_records = engine.read_by_key(&shard_name, "val_key5").await.unwrap();
        assert_eq!(valid_key_records.len(), 1);

        let expired_tag_records = engine
            .read_by_tag(&shard_name, "exp_tag", None, &read_config)
            .await
            .unwrap();
        assert_eq!(expired_tag_records.len(), 0);

        let valid_tag_records = engine
            .read_by_tag(&shard_name, "val_tag", None, &read_config)
            .await
            .unwrap();
        assert_eq!(valid_tag_records.len(), 5);
    }
}
