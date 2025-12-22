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

use super::IndexData;
use crate::core::error::StorageEngineError;
use crate::segment::keys::{key_segment, key_segment_prefix, tag_segment, tag_segment_prefix};
use crate::segment::SegmentIdentity;
use common_base::{error::common::CommonError, utils::serialize};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::engine::engine_save_by_engine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_STORAGE_ENGINE;
use rocksdb_engine::warp::StorageDataWrap;
use std::sync::Arc;

pub struct TagIndexManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl TagIndexManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        TagIndexManager {
            rocksdb_engine_handler,
        }
    }

    pub fn save_tag_position(
        &self,
        segment_iden: &SegmentIdentity,
        tag: String,
        index_data: IndexData,
    ) -> Result<(), StorageEngineError> {
        let key = tag_segment(segment_iden, tag, index_data.offset);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            index_data,
        )?)
    }

    pub async fn get_last_positions_by_tag(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: u64,
        tag: String,
        record_num: u64,
    ) -> Result<Vec<IndexData>, StorageEngineError> {
        let prefix_key = tag_segment_prefix(segment_iden, tag);

        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE)
        {
            cf
        } else {
            return Err(CommonError::RocksDBFamilyNotAvailable(
                DB_COLUMN_FAMILY_STORAGE_ENGINE.to_string(),
            )
            .into());
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(prefix_key.clone());

        let mut results = Vec::new();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    let key = String::from_utf8(key.to_vec())?;
                    if !key.starts_with(&prefix_key) {
                        break;
                    }

                    let data = serialize::deserialize::<StorageDataWrap<IndexData>>(val)?;
                    let index_data = data.data;

                    if index_data.offset < start_offset {
                        iter.next();
                        continue;
                    }

                    results.push(index_data);
                    if results.len() >= (record_num as usize) {
                        break;
                    }
                }
            }
            iter.next();
        }

        Ok(results)
    }

    pub fn save_key_position(
        &self,
        segment_iden: &SegmentIdentity,
        key: String,
        index_data: IndexData,
    ) -> Result<(), StorageEngineError> {
        let key = key_segment(segment_iden, key, index_data.offset);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            index_data,
        )?)
    }

    pub async fn get_last_positions_by_key(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: u64,
        key: String,
        record_num: u64,
    ) -> Result<Vec<IndexData>, StorageEngineError> {
        let prefix_key = key_segment_prefix(segment_iden, key);

        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE)
        {
            cf
        } else {
            return Err(CommonError::RocksDBFamilyNotAvailable(
                DB_COLUMN_FAMILY_STORAGE_ENGINE.to_string(),
            )
            .into());
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(prefix_key.clone());

        let mut results = Vec::new();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    let key = String::from_utf8(key.to_vec())?;
                    if !key.starts_with(&prefix_key) {
                        break;
                    }

                    let data = serialize::deserialize::<StorageDataWrap<IndexData>>(val)?;
                    let index_data = data.data;

                    if index_data.offset < start_offset {
                        iter.next();
                        continue;
                    }

                    results.push(index_data);
                    if results.len() >= (record_num as usize) {
                        break;
                    }
                }
            }
            iter.next();
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::TagIndexManager;
    use crate::{core::test::test_build_segment, segment::index::IndexData};
    use common_base::tools::now_second;
    use rocksdb_engine::test::test_rocksdb_instance;

    #[tokio::test]
    async fn tag_index_test() {
        let rocksdb_engine_handler = test_rocksdb_instance();
        let segment_iden = test_build_segment();
        let tag_index = TagIndexManager::new(rocksdb_engine_handler);

        let timestamp = now_second();

        for i in 0..10 {
            let cur_timestamp = timestamp + i * 10;
            let index_data = IndexData {
                offset: i,
                timestamp: cur_timestamp,
                position: i * 5,
            };
            let tag = format!("tag-{i}");
            let res = tag_index.save_tag_position(&segment_iden, tag, index_data);
            assert!(res.is_ok());
        }

        // get tag + 0
        let tag = format!("tag-{}", 0);
        let res = tag_index
            .get_last_positions_by_tag(&segment_iden, 0, tag, 1000)
            .await;

        assert!(res.is_ok());
        let data = res.unwrap();
        assert!(!data.is_empty());
        let first = data.first().unwrap();
        assert_eq!(first.offset, 0);

        // get tag + 3
        let tag = format!("tag-{}", 3);
        let res = tag_index
            .get_last_positions_by_tag(&segment_iden, 0, tag, 1000)
            .await;

        assert!(res.is_ok());
        let data = res.unwrap();
        assert!(!data.is_empty());
        let first = data.first().unwrap();
        assert_eq!(first.offset, 3);

        // get tag + 7
        let tag = format!("tag-{}", 7);
        let res = tag_index
            .get_last_positions_by_tag(&segment_iden, 0, tag, 1000)
            .await;

        assert!(res.is_ok());
        let data = res.unwrap();
        assert!(!data.is_empty());
        let first = data.first().unwrap();
        assert_eq!(first.offset, 7);
    }

    #[tokio::test]
    async fn key_index_test() {
        let rocksdb_engine_handler = test_rocksdb_instance();
        let segment_iden = test_build_segment();

        let tag_index = TagIndexManager::new(rocksdb_engine_handler);

        let timestamp = now_second();

        for i in 0..10 {
            let cur_timestamp = timestamp + i * 10;
            let index_data = IndexData {
                offset: i,
                timestamp: cur_timestamp,
                position: i * 5,
            };
            let key = format!("key-{i}");
            let res = tag_index.save_key_position(&segment_iden, key, index_data);
            assert!(res.is_ok());
        }

        // get key + 0
        let key = format!("key-{}", 0);
        let res = tag_index
            .get_last_positions_by_key(&segment_iden, 0, key, 1000)
            .await;

        assert!(res.is_ok());
        let data = res.unwrap();
        assert!(!data.is_empty());
        let first = data.first().unwrap();
        assert_eq!(first.offset, 0);

        // get key + 5
        let key = format!("key-{}", 5);
        let res = tag_index
            .get_last_positions_by_key(&segment_iden, 0, key, 1000)
            .await;

        assert!(res.is_ok());
        let data = res.unwrap();
        assert!(!data.is_empty());
        let first = data.first().unwrap();
        assert_eq!(first.offset, 5);

        // get key + 8
        let key = format!("key-{}", 8);
        let res = tag_index
            .get_last_positions_by_key(&segment_iden, 0, key, 1000)
            .await;

        assert!(res.is_ok());
        let data = res.unwrap();
        assert!(!data.is_empty());
        let first = data.first().unwrap();
        assert_eq!(first.offset, 8);
    }
}
