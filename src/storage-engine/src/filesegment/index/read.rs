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
use crate::filesegment::index::build::IndexData;
use crate::filesegment::SegmentIdentity;
use common_base::utils::serialize;
use rocksdb_engine::keys::engine::{
    key_index_key, position_index_key, position_index_prefix, segment_timestamp_index_key,
    segment_timestamp_index_prefix, tag_index_key, tag_index_tag_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

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

pub fn get_in_segment_by_timestamp(
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
    timestamp: i64,
) -> Result<Option<u32>, StorageEngineError> {
    let index = cache_manager.get_offset_index(shard).ok_or_else(|| {
        StorageEngineError::CommonErrorStr(format!("Offset index not found for shard: {}", shard))
    })?;

    Ok(index.find_segment_by_timestamp(timestamp))
}

pub fn get_index_data_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    start_offset: u64,
) -> Result<Option<IndexData>, StorageEngineError> {
    let cf = super::get_storage_cf(rocksdb_engine_handler)?;
    let prefix = position_index_prefix(&segment_iden.shard_name, segment_iden.segment);
    let seek_key = position_index_key(&segment_iden.shard_name, segment_iden.segment, start_offset);

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    iter.seek_for_prev(seek_key.as_bytes());

    if !iter.valid() {
        return Ok(None);
    }
    let (Some(k), Some(v)) = (iter.key(), iter.value()) else {
        return Ok(None);
    };
    if !k.starts_with(prefix.as_bytes()) {
        return Ok(None);
    }
    Ok(Some(serialize::deserialize::<IndexData>(v)?))
}

pub fn get_index_data_by_tag(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    start_offset: Option<u64>,
    tag: &str,
    record_num: usize,
) -> Result<Vec<IndexData>, StorageEngineError> {
    let prefix = tag_index_tag_prefix(shard_name, tag);
    let cf = super::get_storage_cf(rocksdb_engine_handler)?;

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    let seek_key = match start_offset {
        Some(off) => tag_index_key(shard_name, tag, off),
        None => prefix.clone(),
    };
    iter.seek(seek_key.as_bytes());

    let mut results = Vec::new();
    while iter.valid() {
        let (Some(k), Some(v)) = (iter.key(), iter.value()) else {
            iter.next();
            continue;
        };
        if !k.starts_with(prefix.as_bytes()) {
            break;
        }
        results.push(serialize::deserialize::<IndexData>(v)?);
        if results.len() >= record_num {
            break;
        }
        iter.next();
    }

    Ok(results)
}

pub fn get_index_data_by_timestamp(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    start_timestamp: u64,
) -> Result<Option<IndexData>, StorageEngineError> {
    let cf = super::get_storage_cf(rocksdb_engine_handler)?;
    let prefix = segment_timestamp_index_prefix(&segment_iden.shard_name, segment_iden.segment);
    let seek_key = segment_timestamp_index_key(
        &segment_iden.shard_name,
        segment_iden.segment,
        start_timestamp,
    );

    let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
    iter.seek_for_prev(seek_key.as_bytes());

    if !iter.valid() {
        return Ok(None);
    }
    let (Some(k), Some(v)) = (iter.key(), iter.value()) else {
        return Ok(None);
    };
    if !k.starts_with(prefix.as_bytes()) {
        return Ok(None);
    }
    Ok(Some(serialize::deserialize::<IndexData>(v)?))
}

pub fn get_index_data_by_key(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    key: &[u8],
) -> Result<Option<IndexData>, StorageEngineError> {
    let cf = super::get_storage_cf(rocksdb_engine_handler)?;
    let key = key_index_key(shard_name, key);
    Ok(rocksdb_engine_handler.read::<IndexData>(cf, &key)?)
}
