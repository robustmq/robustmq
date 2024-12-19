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
use rocksdb_engine::engine::{rocksdb_engine_get, rocksdb_engine_save};
use rocksdb_engine::warp::StorageDataWrap;
use rocksdb_engine::RocksDBEngine;

use super::keys::{
    offset_segment_end, offset_segment_position, offset_segment_position_prefix,
    offset_segment_start,
};
use super::IndexData;
use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
use crate::core::error::JournalServerError;
use crate::segment::SegmentIdentity;

pub struct OffsetIndexManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl OffsetIndexManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        OffsetIndexManager {
            rocksdb_engine_handler,
        }
    }

    pub fn save_start_offset(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: u64,
    ) -> Result<(), JournalServerError> {
        let key = offset_segment_start(segment_iden);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            start_offset,
        )?)
    }

    pub fn get_start_offset(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<u64, JournalServerError> {
        let key = offset_segment_start(segment_iden);
        if let Some(res) = rocksdb_engine_get(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )? {
            return Ok(serde_json::from_slice::<u64>(&res.data)?);
        }

        Ok(0)
    }

    pub fn save_end_offset(
        &self,
        segment_iden: &SegmentIdentity,
        end_offset: u64,
    ) -> Result<(), JournalServerError> {
        let key = offset_segment_end(segment_iden);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            end_offset,
        )?)
    }

    pub fn get_end_offset(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<u64, JournalServerError> {
        let key = offset_segment_end(segment_iden);
        if let Some(res) = rocksdb_engine_get(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )? {
            return Ok(serde_json::from_slice::<u64>(&res.data)?);
        }

        Ok(0)
    }

    pub fn save_position_offset(
        &self,
        segment_iden: &SegmentIdentity,
        offset: u64,
        index_data: IndexData,
    ) -> Result<(), JournalServerError> {
        let key = offset_segment_position(segment_iden, offset);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            index_data,
        )?)
    }

    //todo Optimize the fetching to avoid scanning the RocksDb index every time
    pub async fn get_last_nearest_position_by_offset(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: u64,
    ) -> Result<u64, JournalServerError> {
        let prefix_key = offset_segment_position_prefix(segment_iden);

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

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(prefix_key.clone());

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

                    return Ok(index_data.position);
                }
            }
            iter.next();
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn offset_index_test() {}
}
