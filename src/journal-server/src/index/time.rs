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
    timestamp_segment_end, timestamp_segment_start, timestamp_segment_time,
    timestamp_segment_time_prefix,
};
use super::IndexData;
use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
use crate::core::error::JournalServerError;
use crate::segment::SegmentIdentity;

pub struct TimestampIndexManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl TimestampIndexManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        TimestampIndexManager {
            rocksdb_engine_handler,
        }
    }

    pub fn save_start_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        start_timestamp: u64,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_start(segment_iden);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            start_timestamp,
        )?)
    }

    pub fn get_start_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<u64, JournalServerError> {
        let key = timestamp_segment_start(segment_iden);
        if let Some(res) = rocksdb_engine_get(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )? {
            return Ok(serde_json::from_slice::<u64>(&res.data)?);
        }

        Ok(0)
    }

    pub fn save_end_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        end_timestamp: u64,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_end(segment_iden);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            end_timestamp,
        )?)
    }

    pub fn get_end_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<u64, JournalServerError> {
        let key = timestamp_segment_end(segment_iden);
        if let Some(res) = rocksdb_engine_get(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )? {
            return Ok(serde_json::from_slice::<u64>(&res.data)?);
        }

        Ok(0)
    }

    pub fn save_timestamp_offset(
        &self,
        segment_iden: &SegmentIdentity,
        timestamp: u64,
        index_data: IndexData,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_time(segment_iden, timestamp);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            index_data,
        )?)
    }

    //todo Optimize the fetching to avoid scanning the RocksDb index every time
    pub async fn get_last_nearest_position_by_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        start_timestamp: u64,
    ) -> Result<Option<IndexData>, JournalServerError> {
        let prefix_key = timestamp_segment_time_prefix(segment_iden);

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

        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    let key = String::from_utf8(key.to_vec())?;
                    if !key.starts_with(&prefix_key) {
                        break;
                    }

                    let data = serde_json::from_slice::<StorageDataWrap>(val)?;
                    let index_data = serde_json::from_slice::<IndexData>(data.data.as_ref())?;

                    if index_data.offset < start_timestamp {
                        continue;
                    }

                    return Ok(Some(index_data));
                }
            }
            iter.next();
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp_index_test() {}
}
