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

use crate::{commitlog::rocksdb::engine::RocksDBStorageEngine, core::error::StorageEngineError};
use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::{loop_select_ticket, now_second},
    utils::serialize::deserialize,
};
use common_config::{broker::broker_config, storage::StorageType};
use metadata_struct::storage::{record::StorageRecord, shard::EngineShard};
use rocksdb::WriteBatch;
use rocksdb_engine::keys::engine::{
    key_index_key, record_key, record_prefix, tag_index_key, timestamp_index_key,
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
        let shard_infos: Vec<EngineShard> = self
            .cache_manager
            .shards
            .iter()
            .filter(|e| e.value().config.storage_type == StorageType::EngineRocksDB)
            .map(|e| e.value().clone())
            .collect();

        if shard_infos.is_empty() {
            return Ok(());
        }

        // chunk
        let task_num = broker_config().storage_runtime.expire_scan_task_num;

        let chunk_size = shard_infos.len().div_ceil(task_num);
        let chunks: Vec<Vec<EngineShard>> = shard_infos
            .chunks(chunk_size.max(1))
            .map(|c| c.to_vec())
            .collect();

        // message clear task
        let mut handles = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let engine = self.clone();
            handles.push(tokio::spawn(async move {
                for shard in chunk {
                    engine.scan_and_delete_data_by_shard(shard).await?;
                }
                Ok::<(), StorageEngineError>(())
            }));
        }

        for handle in handles {
            handle
                .await
                .map_err(|e| CommonError::CommonError(e.to_string()))??;
        }

        Ok(())
    }

    // Single forward pass: delete the expired prefix (create_t older than the
    // retention cutoff) plus its indices, stopping at the first live record.
    async fn scan_and_delete_data_by_shard(
        &self,
        shard: EngineShard,
    ) -> Result<(), StorageEngineError> {
        let earliest_timestamp = now_second().saturating_sub(shard.config.retention_sec);
        let earliest_offset = self
            .commitlog_offset
            .get_earliest_offset(&shard.shard_name)?;
        let cf = self.get_cf()?;

        let prefix = record_prefix(&shard.shard_name, 0);
        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(record_key(&shard.shard_name, 0, earliest_offset).as_bytes());

        const FLUSH_EVERY: u64 = 1000;
        let mut batch = WriteBatch::default();
        let mut pending = 0u64;
        let mut new_earliest = earliest_offset;

        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };
            if !key_bytes.starts_with(prefix.as_bytes()) {
                break;
            }
            let Some(value) = iter.value() else {
                break;
            };
            let Ok(record) = deserialize::<StorageRecord>(value) else {
                iter.next();
                continue;
            };

            if record.metadata.create_t >= earliest_timestamp {
                break;
            }

            let offset = record.metadata.offset;
            batch.delete_cf(&cf, key_bytes);
            if let Some(key) = &record.metadata.key {
                batch.delete_cf(&cf, key_index_key(&shard.shard_name, key));
            }
            if let Some(tags) = &record.metadata.tags {
                for tag in tags.iter() {
                    batch.delete_cf(
                        &cf,
                        tag_index_key(&shard.shard_name, tag, offset).as_bytes(),
                    );
                }
            }
            if offset.is_multiple_of(5000) && record.metadata.create_t > 0 {
                batch.delete_cf(
                    &cf,
                    timestamp_index_key(&shard.shard_name, record.metadata.create_t, offset)
                        .as_bytes(),
                );
            }
            new_earliest = offset + 1;

            pending += 1;
            if pending >= FLUSH_EVERY {
                self.rocksdb_engine_handler
                    .write_batch(std::mem::take(&mut batch))?;
                pending = 0;
            }
            iter.next();
        }

        if pending > 0 {
            self.rocksdb_engine_handler.write_batch(batch)?;
        }
        if new_earliest > earliest_offset {
            self.commitlog_offset
                .save_earliest_offset(&shard.shard_name, new_earliest)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        commitlog::rocksdb::engine::RocksDBStorageEngine,
        core::{cache::StorageCacheManager, offset::ShardOffset},
    };
    use broker_core::cache::NodeCacheManager;
    use common_base::uuid::unique_id;
    use common_config::config::BrokerConfig;
    use metadata_struct::storage::{
        adapter_read_config::AdapterReadConfig, adapter_record::AdapterWriteRecord,
        shard::EngineShard,
    };
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_scan_and_delete_expire_data() {
        use common_base::tools::now_second;
        use common_config::storage::StorageType;
        use metadata_struct::storage::record::StorageRecord;
        use metadata_struct::storage::shard::EngineShardConfig;
        use rocksdb_engine::keys::engine::record_key;

        let shard_name = unique_id();
        let db = test_rocksdb_instance();
        let cache_manager = Arc::new(StorageCacheManager::new(Arc::new(NodeCacheManager::new(
            BrokerConfig::default(),
        ))));
        let commit_offset = ShardOffset::new(cache_manager.clone(), db.clone());
        commit_offset.save_earliest_offset(&shard_name, 0).unwrap();
        commit_offset.save_latest_offset(&shard_name, 0).unwrap();

        let engine = RocksDBStorageEngine::new(cache_manager.clone(), db);
        cache_manager.set_shard(EngineShard {
            shard_name: shard_name.clone(),
            config: EngineShardConfig {
                storage_type: StorageType::EngineRocksDB,
                retention_sec: 100,
                ..Default::default()
            },
            ..Default::default()
        });

        let messages: Vec<AdapterWriteRecord> = (0..10)
            .map(|i| AdapterWriteRecord {
                key: Some(format!("key{i}").into()),
                tags: Some(vec![format!("t{i}")]),
                ..Default::default()
            })
            .collect();
        engine.batch_write(&shard_name, &messages).await.unwrap();

        let cf = engine.get_cf().unwrap();
        let old_ts = now_second() - 200;
        for off in 0..3u64 {
            let key = record_key(&shard_name, 0, off);
            let mut record = engine
                .rocksdb_engine_handler
                .read::<StorageRecord>(cf.clone(), &key)
                .unwrap()
                .unwrap();
            record.metadata.create_t = old_ts;
            engine
                .rocksdb_engine_handler
                .write(cf.clone(), &key, &record)
                .unwrap();
        }

        engine.scan_and_delete_expire_data().await.unwrap();

        assert_eq!(
            engine
                .commitlog_offset
                .get_earliest_offset(&shard_name)
                .unwrap(),
            3
        );
        let read_config = AdapterReadConfig {
            max_record_num: 100,
            max_size: 1024 * 1024,
        };
        let records = engine
            .read_by_offset(&shard_name, 0, &read_config)
            .await
            .unwrap();
        assert_eq!(records.len(), 7);
        assert_eq!(records[0].metadata.offset, 3);
        assert!(engine
            .read_by_key(&shard_name, b"key0")
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            engine
                .read_by_key(&shard_name, b"key5")
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(engine
            .read_by_tag(&shard_name, "t0", None, &read_config)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            engine
                .read_by_tag(&shard_name, "t5", None, &read_config)
                .await
                .unwrap()
                .len(),
            1
        );
    }
}
