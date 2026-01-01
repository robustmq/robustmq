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

use common_base::utils::serialize::deserialize;
use metadata_struct::storage::{
    adapter_read_config::AdapterReadConfig, storage_record::StorageRecord,
};
use rocksdb_engine::keys::storage::{key_index_key, shard_record_key, tag_index_tag_prefix};

use crate::{
    core::error::StorageEngineError,
    rocksdb::engine::{IndexInfo, RocksDBStorageEngine},
};

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
impl RocksDBStorageEngine {
    pub async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let cf = self.get_cf()?;

        let keys: Vec<String> = (offset..offset.saturating_add(read_config.max_record_num))
            .map(|i| shard_record_key(shard, i))
            .collect();

        let mut records = Vec::new();
        let mut total_size = 0;

        let batch_results = self
            .rocksdb_engine_handler
            .multi_get::<StorageRecord>(cf, &keys)?;
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
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let cf = self.get_cf()?;
        let tag_offset_key_prefix = tag_index_tag_prefix(shard, tag);
        let tag_entries = self
            .rocksdb_engine_handler
            .read_prefix(cf.clone(), &tag_offset_key_prefix)?;

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
        let batch_results = self
            .rocksdb_engine_handler
            .multi_get::<StorageRecord>(cf, &keys)?;
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
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let cf = self.get_cf()?;
        let key_index = key_index_key(shard, key);

        let key_offset_bytes = match self.rocksdb_engine_handler.db.get_cf(&cf, &key_index) {
            Ok(Some(data)) => data,
            Ok(_) => return Ok(Vec::new()),
            Err(e) => {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Failed to read key offset: {e:?}"
                )))
            }
        };

        let index = deserialize::<IndexInfo>(&key_offset_bytes)?;

        let shard_record_key = shard_record_key(shard, index.offset);
        let Some(record) = self
            .rocksdb_engine_handler
            .read::<StorageRecord>(cf, &shard_record_key)?
        else {
            return Ok(Vec::new());
        };

        Ok(vec![record])
    }
}
