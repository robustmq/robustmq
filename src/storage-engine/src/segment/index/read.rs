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

use crate::core::error::StorageEngineError;
use crate::segment::index::build::IndexData;
use crate::segment::keys::{
    key_segment, offset_segment_position_prefix, tag_segment_prefix, timestamp_segment_time_prefix,
};
use crate::segment::SegmentIdentity;
use common_base::{error::common::CommonError, utils::serialize};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_STORAGE_ENGINE;
use std::sync::Arc;

//todo Optimize the fetching to avoid scanning the RocksDb index every time
pub async fn get_index_data_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    start_offset: u64,
) -> Result<Option<IndexData>, StorageEngineError> {
    let prefix_key = offset_segment_position_prefix(segment_iden);

    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            DB_COLUMN_FAMILY_STORAGE_ENGINE.to_string(),
        )
        .into());
    };

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    iter.seek(prefix_key.clone());

    while iter.valid() {
        if let Some(key) = iter.key() {
            if let Some(val) = iter.value() {
                let key = String::from_utf8(key.to_vec())?;
                if !key.starts_with(&prefix_key) {
                    break;
                }

                let index_data = serialize::deserialize::<IndexData>(val)?;

                if index_data.offset < start_offset {
                    iter.next();
                    continue;
                }

                return Ok(Some(index_data));
            }
        }
        iter.next();
    }

    Ok(None)
}

pub async fn get_index_data_by_tag(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    start_offset: u64,
    tag: String,
    record_num: u64,
) -> Result<Vec<IndexData>, StorageEngineError> {
    let prefix_key = tag_segment_prefix(segment_iden, tag);

    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            DB_COLUMN_FAMILY_STORAGE_ENGINE.to_string(),
        )
        .into());
    };

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    iter.seek(prefix_key.clone());

    let mut results = Vec::new();
    while iter.valid() {
        if let Some(key) = iter.key() {
            if let Some(val) = iter.value() {
                let key = String::from_utf8(key.to_vec())?;
                if !key.starts_with(&prefix_key) {
                    break;
                }

                let index_data = serialize::deserialize::<IndexData>(val)?;

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

//todo Optimize the fetching to avoid scanning the RocksDb index every time
pub async fn get_index_data_by_timestamp(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    start_timestamp: u64,
) -> Result<Option<IndexData>, StorageEngineError> {
    let prefix_key = timestamp_segment_time_prefix(segment_iden);

    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            DB_COLUMN_FAMILY_STORAGE_ENGINE.to_string(),
        )
        .into());
    };

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    iter.seek(prefix_key.clone());

    while iter.valid() {
        if let Some(key) = iter.key() {
            if let Some(val) = iter.value() {
                let key = String::from_utf8(key.to_vec())?;
                if !key.starts_with(&prefix_key) {
                    break;
                }

                let index_data = serialize::deserialize::<IndexData>(val)?;

                if index_data.timestamp < start_timestamp {
                    iter.next();
                    continue;
                }

                return Ok(Some(index_data));
            }
        }
        iter.next();
    }

    Ok(None)
}

pub async fn get_index_data_positions_by_key(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    key: String,
) -> Result<Option<IndexData>, StorageEngineError> {
    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            DB_COLUMN_FAMILY_STORAGE_ENGINE.to_string(),
        )
        .into());
    };

    let key = key_segment(segment_iden, key);
    Ok(rocksdb_engine_handler.read::<IndexData>(cf, &key)?)
}
