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

use common_base::tools::now_second;
use common_base::utils::serialize::{deserialize, serialize};
use common_base::{error::common::CommonError, utils::serialize};
use dashmap::DashMap;
use metadata_struct::adapter::{read_config::ReadConfig, record::StorageAdapterRecord};
use metadata_struct::adapter::{ShardInfo, ShardOffset};
use metadata_struct::storage::convert::convert_adapter_record_to_engine;
use metadata_struct::storage::record::StorageEngineRecord;
use rocksdb::WriteBatch;
use rocksdb_engine::keys::storage::*;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_BROKER;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize, Clone)]
struct OffsetInfo {
    pub group_name: String,
    pub shard_name: String,
    pub offset: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct IndexInfo {
    pub shard_name: String,
    pub offset: u64,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct RocksDBStorageEngine {
    pub db: Arc<RocksDBEngine>,
    shard_write_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
}

impl RocksDBStorageEngine {
    pub fn new(db: Arc<RocksDBEngine>) -> Self {
        RocksDBStorageEngine {
            db,
            shard_write_locks: DashMap::new(),
        }
    }

    fn get_cf(&self) -> Result<Arc<rocksdb::BoundColumnFamily<'_>>, Box<CommonError>> {
        self.db.cf_handle(DB_COLUMN_FAMILY_BROKER).ok_or_else(|| {
            Box::new(CommonError::CommonError(format!(
                "Column family '{}' not found",
                DB_COLUMN_FAMILY_BROKER
            )))
        })
    }

    fn save_latest_offset(&self, shard_name: &str, offset: u64) -> Result<(), Box<CommonError>> {
        let cf = self.get_cf()?;
        let key = latest_offset_key(shard_name);
        self.db.write(cf, &key, &offset).map_err(Box::new)?;
        Ok(())
    }

    fn get_latest_offset(&self, shard_name: &str) -> Result<u64, Box<CommonError>> {
        let cf = self.get_cf()?;
        let key = latest_offset_key(shard_name);
        Ok(self
            .db
            .read::<u64>(cf, &key)
            .map_err(Box::new)?
            .unwrap_or(0))
    }

    fn _save_earliest_offset(&self, shard_name: &str, offset: u64) -> Result<(), Box<CommonError>> {
        let cf = self.get_cf()?;
        let key = earliest_offset_key(shard_name);
        if !self.db.exist(cf.clone(), &key) {
            self.db.write(cf, &key, &offset).map_err(Box::new)?;
        }
        Ok(())
    }

    fn _get_earliest_offset(&self, shard_name: &str) -> Result<u64, Box<CommonError>> {
        let cf = self.get_cf()?;
        let key = earliest_offset_key(shard_name);
        Ok(self
            .db
            .read::<u64>(cf, &key)
            .map_err(Box::new)?
            .unwrap_or(0))
    }

    async fn batch_write_internal(
        &self,
        shard_name: &str,
        messages: &[StorageAdapterRecord],
    ) -> Result<Vec<u64>, CommonError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let lock = self
            .shard_write_locks
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();

        let _guard = lock.lock().await;

        let cf = self.get_cf().map_err(|e| *e)?;
        let mut offset = self.get_latest_offset(shard_name).map_err(|e| *e)?;

        let mut offset_res = Vec::with_capacity(messages.len());
        let mut batch = WriteBatch::default();

        for msg in messages {
            offset_res.push(offset);

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

        self.db.write_batch(batch)?;
        self.save_latest_offset(shard_name, offset)
            .map_err(|e| *e)?;
        Ok(offset_res)
    }

    async fn search_index_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<IndexInfo>, CommonError> {
        let cf = self.get_cf().map_err(|e| *e)?;
        let timestamp_index_prefix = timestamp_index_prefix(shard);
        let mut iter = self.db.db.raw_iterator_cf(&cf);
        iter.seek(&timestamp_index_prefix);

        let mut last_index = None;
        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };

            let Some(value_byte) = iter.value() else {
                break;
            };

            let key = match String::from_utf8(key_bytes.to_vec()) {
                Ok(k) => k,
                Err(_) => {
                    iter.next();
                    continue;
                }
            };

            if !key.starts_with(&timestamp_index_prefix) {
                break;
            }

            let index = deserialize::<IndexInfo>(value_byte)?;

            if last_index.is_none() {
                last_index = Some(index.clone());
            }

            if index.create_time > timestamp {
                return Ok(last_index);
            }
            last_index = Some(index);

            iter.next();
        }

        Ok(last_index)
    }

    async fn read_data_by_time(
        &self,
        shard: &str,
        start_index: &Option<IndexInfo>,
        timestamp: u64,
    ) -> Result<Option<u64>, CommonError> {
        let cf = self.get_cf().map_err(|e| *e)?;
        let seek_key = if let Some(si) = start_index {
            shard_record_key(shard, si.offset)
        } else {
            shard_record_key_prefix(shard)
        };
        let prefix = shard_record_key_prefix(shard);

        let mut iter = self.db.db.raw_iterator_cf(&cf);
        iter.seek(&seek_key);

        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };

            let Some(value_byte) = iter.value() else {
                break;
            };

            let key = match String::from_utf8(key_bytes.to_vec()) {
                Ok(k) => k,
                Err(_) => {
                    iter.next();
                    continue;
                }
            };

            if !key.starts_with(&prefix) {
                break;
            }

            if let Ok(engine_record) = deserialize::<StorageEngineRecord>(value_byte) {
                if engine_record.metadata.create_t >= timestamp {
                    return Ok(Some(engine_record.metadata.offset));
                }
            }

            iter.next();
        }

        Ok(None)
    }
}

impl RocksDBStorageEngine {
    pub async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        let shard_name = &shard.shard_name;
        let cf = self.get_cf().map_err(|e| *e)?;
        let shard_info_key = shard_info_key(shard_name);

        if self.db.exist(cf.clone(), &shard_info_key) {
            return Err(CommonError::CommonError(format!(
                "shard {shard_name} already exists"
            )));
        }

        // init shard
        self.db.write(cf.clone(), &shard_info_key, &shard)?;

        // init shard offset
        self.db
            .write(cf.clone(), &earliest_offset_key(shard_name), &0_u64)?;
        self.db
            .write(cf.clone(), &latest_offset_key(shard_name), &0_u64)?;

        // init shard lock
        self.shard_write_locks
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())));

        Ok(())
    }

    pub async fn list_shard(&self, shard: Option<String>) -> Result<Vec<ShardInfo>, CommonError> {
        let cf = self.get_cf().map_err(|e| *e)?;
        if let Some(shard_name) = shard {
            let key = shard_info_key(&shard_name);
            if let Some(v) = self.db.read::<ShardInfo>(cf.clone(), &key)? {
                Ok(vec![v])
            } else {
                Ok(Vec::new())
            }
        } else {
            let raw_shard_info = self.db.read_prefix(cf.clone(), &shard_info_key_prefix())?;
            let mut result = Vec::new();
            for (_, v) in raw_shard_info {
                result.push(serialize::deserialize::<ShardInfo>(v.as_slice())?);
            }
            Ok(result)
        }
    }

    pub async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        let cf = self.get_cf().map_err(|e| *e)?;
        let shard_info_key = shard_info_key(shard);

        if !self.db.exist(cf.clone(), &shard_info_key) {
            return Err(CommonError::CommonError(format!(
                "shard {shard} does not exist"
            )));
        }

        // delete records
        let record_prefix = shard_record_key_prefix(shard);
        self.db.delete_prefix(cf.clone(), &record_prefix)?;

        // delete key index
        let key_prefix = key_index_prefix(shard);
        self.db.delete_prefix(cf.clone(), &key_prefix)?;

        // delete tag index
        let tag_index_prefix = tag_index_prefix(shard);
        self.db.delete_prefix(cf.clone(), &tag_index_prefix)?;

        // delete timestamp index
        let timestamp_index_prefix = timestamp_index_prefix(shard);
        self.db.delete_prefix(cf.clone(), &timestamp_index_prefix)?;

        // delete shard offset
        self.db.delete(cf.clone(), &earliest_offset_key(shard))?;
        self.db.delete(cf.clone(), &latest_offset_key(shard))?;

        // delete shard info
        self.db.delete(cf, &shard_info_key)?;

        // remove lock
        self.shard_write_locks.remove(shard);

        Ok(())
    }

    pub async fn write(
        &self,
        shard: &str,
        message: &StorageAdapterRecord,
    ) -> Result<u64, CommonError> {
        let offsets = self
            .batch_write_internal(shard, std::slice::from_ref(message))
            .await?;

        offsets
            .first()
            .cloned()
            .ok_or_else(|| CommonError::CommonError("Empty offset result from write".to_string()))
    }

    pub async fn batch_write(
        &self,
        shard: &str,
        messages: &[StorageAdapterRecord],
    ) -> Result<Vec<u64>, CommonError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        self.batch_write_internal(shard, messages).await
    }

    pub async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<StorageEngineRecord>, CommonError> {
        let cf = self.get_cf().map_err(|e| *e)?;

        let keys: Vec<String> = (offset..offset.saturating_add(read_config.max_record_num))
            .map(|i| shard_record_key(shard, i))
            .collect();

        let mut records = Vec::new();
        let mut total_size = 0;

        let batch_results = self.db.multi_get::<StorageEngineRecord>(cf, &keys)?;
        for record_opt in batch_results {
            let Some(record) = record_opt else {
                break;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record);
        }

        Ok(records)
    }

    pub async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &ReadConfig,
    ) -> Result<Vec<StorageEngineRecord>, CommonError> {
        let cf = self.get_cf().map_err(|e| *e)?;
        let tag_offset_key_prefix = tag_index_tag_prefix(shard, tag);
        let tag_entries = self.db.read_prefix(cf.clone(), &tag_offset_key_prefix)?;

        // Filter and collect offsets >= specified offset
        let mut offsets = Vec::new();
        for (_key, value) in tag_entries {
            let record_offset = deserialize::<IndexInfo>(&value)?;

            if let Some(so) = start_offset {
                if record_offset.offset < so {
                    continue;
                }
            }

            offsets.push(record_offset.offset);
            if offsets.len() >= read_config.max_record_num as usize {
                break;
            }
        }

        if offsets.is_empty() {
            return Ok(Vec::new());
        }

        // Build record keys from offsets
        let keys: Vec<String> = offsets
            .iter()
            .map(|off| shard_record_key(shard, *off))
            .collect();

        // Batch read records
        let batch_results = self.db.multi_get::<StorageEngineRecord>(cf, &keys)?;
        let mut records = Vec::new();
        let mut total_size = 0;

        for record_opt in batch_results {
            let Some(record) = record_opt else {
                continue;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record);
        }

        Ok(records)
    }

    pub async fn read_by_key(
        &self,
        shard: &str,
        key: &str,
    ) -> Result<Vec<StorageEngineRecord>, CommonError> {
        let cf = self.get_cf().map_err(|e| *e)?;
        let key_index = key_index_key(shard, key);

        let key_offset_bytes = match self.db.db.get_cf(&cf, &key_index) {
            Ok(Some(data)) => data,
            Ok(_) => return Ok(Vec::new()),
            Err(e) => {
                return Err(CommonError::CommonError(format!(
                    "Failed to read key offset: {e:?}"
                )))
            }
        };

        let index = deserialize::<IndexInfo>(&key_offset_bytes)?;

        let shard_record_key = shard_record_key(shard, index.offset);
        let Some(record) = self.db.read::<StorageEngineRecord>(cf, &shard_record_key)? else {
            return Ok(Vec::new());
        };

        Ok(vec![record])
    }

    pub async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let index: Option<IndexInfo> = self.search_index_by_timestamp(shard, timestamp).await?;
        if let Some(idx) = index {
            if let Some(found_offset) = self
                .read_data_by_time(shard, &Some(idx.clone()), timestamp)
                .await?
            {
                return Ok(Some(ShardOffset {
                    shard_name: shard.to_string(),
                    offset: found_offset,
                    ..Default::default()
                }));
            }
        }
        Ok(None)
    }

    pub async fn get_offset_by_group(
        &self,
        group_name: &str,
    ) -> Result<Vec<ShardOffset>, CommonError> {
        let cf = self.get_cf().map_err(|e| *e)?;
        let group_record_offsets_key_prefix = group_record_offsets_key_prefix(group_name);

        let mut offsets = Vec::new();
        for (_, v) in self.db.read_prefix(cf, &group_record_offsets_key_prefix)? {
            let info = deserialize::<OffsetInfo>(&v)?;
            offsets.push(ShardOffset {
                group: info.group_name,
                shard_name: info.shard_name,
                offset: info.offset,
                ..Default::default()
            });
        }

        Ok(offsets)
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        offsets: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        if offsets.is_empty() {
            return Ok(());
        }

        let cf = self.get_cf().map_err(|e| *e)?;
        let mut batch = WriteBatch::default();

        for (shard_name, offset) in offsets.iter() {
            let group_record_offsets_key = group_record_offsets_key(group_name, shard_name);
            let info = OffsetInfo {
                group_name: group_name.to_string(),
                shard_name: shard_name.to_string(),
                offset: *offset,
            };
            batch.put_cf(&cf, group_record_offsets_key.as_bytes(), serialize(&info)?);
        }

        self.db.write_batch(batch)?;

        Ok(())
    }
}
