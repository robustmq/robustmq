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

use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::segment::index::build::IndexData;
use crate::segment::SegmentIdentity;
use common_base::{error::common::CommonError, utils::serialize};
use rocksdb_engine::keys::engine::{
    engine_key_index, offset_segment_position_prefix, tag_segment_prefix,
    timestamp_segment_time_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_STORAGE_ENGINE;
use std::sync::Arc;

fn get_storage_cf(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<Arc<rocksdb::BoundColumnFamily<'_>>, StorageEngineError> {
    rocksdb_engine_handler
        .cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE)
        .ok_or_else(|| {
            CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_STORAGE_ENGINE.to_string())
                .into()
        })
}

pub fn get_in_segment_by_offset(
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
    offset: u64,
) -> Result<Option<u32>, StorageEngineError> {
    let index = cache_manager.get_offset_index(shard).ok_or_else(|| {
        StorageEngineError::CommonErrorStr(format!("Offset index not found for shard: {}", shard))
    })?;

    let offset_i64 = offset as i64;
    Ok(index.find_segment(offset_i64))
}

pub fn get_index_data_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    start_offset: u64,
) -> Result<Option<IndexData>, StorageEngineError> {
    let prefix_key = offset_segment_position_prefix(&segment_iden.shard_name, segment_iden.segment);
    let cf = get_storage_cf(rocksdb_engine_handler)?;

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    iter.seek(&prefix_key);

    let mut last_valid_index = None;

    while iter.valid() {
        if let Some(key) = iter.key() {
            if let Some(val) = iter.value() {
                let key = String::from_utf8(key.to_vec())?;
                if !key.starts_with(&prefix_key) {
                    break;
                }

                let index_data = serialize::deserialize::<IndexData>(val)?;

                if index_data.offset <= start_offset {
                    last_valid_index = Some(index_data);
                    iter.next();
                } else {
                    break;
                }
            } else {
                iter.next();
            }
        } else {
            iter.next();
        }
    }

    Ok(last_valid_index)
}

pub fn get_index_data_by_tag(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    start_offset: Option<u64>,
    tag: &str,
    record_num: usize,
) -> Result<Vec<IndexData>, StorageEngineError> {
    let prefix_key = tag_segment_prefix(&segment_iden.shard_name, segment_iden.segment, tag);
    let cf = get_storage_cf(rocksdb_engine_handler)?;

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    iter.seek(&prefix_key);

    let mut results = Vec::new();
    while iter.valid() {
        if let Some(key) = iter.key() {
            if let Some(val) = iter.value() {
                let key = String::from_utf8(key.to_vec())?;
                if !key.starts_with(&prefix_key) {
                    break;
                }

                let index_data = serialize::deserialize::<IndexData>(val)?;

                if let Some(st) = start_offset {
                    if index_data.offset < st {
                        iter.next();
                        continue;
                    }
                }

                results.push(index_data);
                if results.len() >= record_num {
                    break;
                }
                iter.next();
            } else {
                iter.next();
            }
        } else {
            iter.next();
        }
    }

    Ok(results)
}

pub fn get_index_data_by_timestamp(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    start_timestamp: u64,
) -> Result<Option<IndexData>, StorageEngineError> {
    let prefix_key = timestamp_segment_time_prefix(&segment_iden.shard_name, segment_iden.segment);
    let cf = get_storage_cf(rocksdb_engine_handler)?;

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    iter.seek(&prefix_key);

    let mut last_valid_index = None;

    while iter.valid() {
        if let Some(key) = iter.key() {
            if let Some(val) = iter.value() {
                let key = String::from_utf8(key.to_vec())?;
                if !key.starts_with(&prefix_key) {
                    break;
                }

                let index_data = serialize::deserialize::<IndexData>(val)?;

                if index_data.timestamp <= start_timestamp {
                    last_valid_index = Some(index_data);
                    iter.next();
                } else {
                    break;
                }
            } else {
                iter.next();
            }
        } else {
            iter.next();
        }
    }

    Ok(last_valid_index)
}

pub fn get_index_data_by_key(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    key: String,
) -> Result<Option<IndexData>, StorageEngineError> {
    let cf = get_storage_cf(rocksdb_engine_handler)?;
    let key = engine_key_index(&segment_iden.shard_name, segment_iden.segment, key);
    Ok(rocksdb_engine_handler.read::<IndexData>(cf, &key)?)
}
