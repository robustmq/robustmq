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

use common_base::error::common::CommonError;
use rocksdb_engine::engine::rocksdb_engine_save;
use rocksdb_engine::warp::StorageDataWrap;
use rocksdb_engine::RocksDBEngine;

use super::keys::{key_segment, tag_segment, tag_segment_prefix};
use super::IndexData;
use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
use crate::core::error::JournalServerError;
use crate::segment::SegmentIdentity;

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
    ) -> Result<(), JournalServerError> {
        let key = tag_segment(segment_iden, tag, index_data.offset);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            index_data,
        )?)
    }

    pub async fn get_last_positions_by_tag(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: u64,
        tag: String,
        record_num: u64,
    ) -> Result<Vec<IndexData>, JournalServerError> {
        let prefix_key = tag_segment_prefix(segment_iden, tag);

        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_INDEX)
        {
            cf
        } else {
            return Err(
                CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_INDEX.to_string()).into(),
            );
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(cf);
        iter.seek(prefix_key.clone());

        let mut results = Vec::new();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    let key = String::from_utf8(key.to_vec())?;
                    if !key.starts_with(&prefix_key) {
                        break;
                    }

                    let data = serde_json::from_slice::<StorageDataWrap>(val)?;
                    let index_data = serde_json::from_slice::<IndexData>(data.data.as_ref())?;

                    if index_data.offset < start_offset {
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
    ) -> Result<(), JournalServerError> {
        let key = key_segment(segment_iden, key, index_data.offset);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            index_data,
        )?)
    }

    pub async fn get_last_positions_by_key(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: u64,
        key: String,
        record_num: u64,
    ) -> Result<Vec<IndexData>, JournalServerError> {
        let prefix_key = tag_segment_prefix(segment_iden, key);

        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_INDEX)
        {
            cf
        } else {
            return Err(
                CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_INDEX.to_string()).into(),
            );
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(cf);
        iter.seek(prefix_key.clone());

        let mut results = Vec::new();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    let key = String::from_utf8(key.to_vec())?;
                    if !key.starts_with(&prefix_key) {
                        break;
                    }

                    let data = serde_json::from_slice::<StorageDataWrap>(val)?;
                    let index_data = serde_json::from_slice::<IndexData>(data.data.as_ref())?;

                    if index_data.offset < start_offset {
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
    #[test]
    fn timestamp_index_test() {}
}
