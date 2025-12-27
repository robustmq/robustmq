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
use common_base::tools::now_second;
use rocksdb_engine::keys::meta::{key_offset, key_offset_by_group};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::base::{batch_encode_data, get_cf_handle};
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_META_DATA;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_prefix_list_by_meta_data,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct OffsetData {
    pub group: String,
    pub shard_name: String,
    pub offset: u64,
    pub timestamp: u64,
}

pub struct OffsetStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl OffsetStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        OffsetStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(&self, offsets: &[OffsetData]) -> Result<(), CommonError> {
        if offsets.is_empty() {
            return Ok(());
        }

        let mut batch = rocksdb::WriteBatch::default();
        let cf = get_cf_handle(&self.rocksdb_engine_handler, DB_COLUMN_FAMILY_META_DATA)?;
        for offset in offsets {
            let key = key_offset(&offset.group, &offset.shard_name);

            let offset_data = OffsetData {
                group: offset.group.clone(),
                shard_name: offset.shard_name.clone(),
                offset: offset.offset,
                timestamp: now_second(),
            };
            batch.put_cf(&cf, key, &batch_encode_data(offset_data)?);
        }
        self.rocksdb_engine_handler.db.write(batch)?;
        Ok(())
    }

    pub fn delete(&self, group: &str, shard_name: &str) -> Result<(), CommonError> {
        let key = key_offset(group, shard_name);
        engine_delete_by_meta_data(&self.rocksdb_engine_handler, &key)
    }

    pub fn group_offset(&self, group: &str) -> Result<Vec<OffsetData>, CommonError> {
        let prefix_key = key_offset_by_group(group);

        let data = engine_prefix_list_by_meta_data::<OffsetData>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;

        Ok(data.into_iter().map(|row| row.data).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn create_offset_data(group: &str, shard: &str, offset: u64) -> OffsetData {
        OffsetData {
            group: group.to_string(),
            shard_name: shard.to_string(),
            offset,
            timestamp: now_second(),
        }
    }

    #[test]
    fn test_offset_batch_save() {
        let storage = OffsetStorage::new(test_rocksdb_instance());
        let group = "group1";

        // Batch save two offsets
        let offsets = vec![
            create_offset_data(group, "shard1", 100),
            create_offset_data(group, "shard2", 200),
        ];
        storage.save(&offsets).unwrap();

        // Verify
        let list = storage.group_offset(group).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|o| o.offset == 100));
        assert!(list.iter().any(|o| o.offset == 200));
    }

    #[test]
    fn test_offset_delete() {
        let storage = OffsetStorage::new(test_rocksdb_instance());
        let group = "group1";

        // Save
        let offsets = vec![
            create_offset_data(group, "shard1", 100),
            create_offset_data(group, "shard2", 200),
        ];
        storage.save(&offsets).unwrap();
        assert_eq!(storage.group_offset(group).unwrap().len(), 2);

        // Delete one
        storage.delete(group, "shard1").unwrap();

        let remaining = storage.group_offset(group).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].offset, 200);
    }

    #[test]
    fn test_group_offset_empty() {
        let storage = OffsetStorage::new(test_rocksdb_instance());
        let list = storage.group_offset("group1").unwrap();
        assert!(list.is_empty());

        let empty: Vec<OffsetData> = vec![];
        storage.save(&empty).unwrap();
    }
}
