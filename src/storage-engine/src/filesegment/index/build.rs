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
use crate::filesegment::SegmentIdentity;
use common_base::utils::serialize::{deserialize, serialize};
use rocksdb::WriteBatch;
use rocksdb_engine::keys::engine::{
    key_index_key, key_index_prefix, position_index_key, segment_prefix,
    segment_timestamp_index_key, tag_index_key, tag_index_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IndexData {
    pub segment: u32,
    pub offset: u64,
    pub timestamp: u64,
    pub position: u64,
}

#[derive(Default, Clone)]
pub enum IndexTypeEnum {
    #[default]
    Offset,
    Tag,
    Key,
    Time,
}

#[derive(Default, Clone)]
pub struct BuildIndexRaw {
    pub index_type: IndexTypeEnum,
    pub key: Option<bytes::Bytes>,
    pub tag: Option<String>,
    pub timestamp: Option<u64>,
    pub offset: u64,
}

pub fn save_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    index_data: &[BuildIndexRaw],
    offset_positions: &HashMap<u64, u64>,
) -> Result<(), StorageEngineError> {
    if index_data.is_empty() {
        return Ok(());
    }

    let cf = super::get_storage_cf(rocksdb_engine_handler)?;

    let mut batch = WriteBatch::default();
    for data in index_data.iter() {
        let position = if let Some(position) = offset_positions.get(&data.offset) {
            *position
        } else {
            continue;
        };

        match data.index_type {
            IndexTypeEnum::Offset => {
                let index_data = IndexData {
                    segment: segment_iden.segment,
                    offset: data.offset,
                    position,
                    timestamp: 0,
                };
                let serialized_data = serialize(&index_data)?;
                let key =
                    position_index_key(&segment_iden.shard_name, segment_iden.segment, data.offset);
                batch.put_cf(&cf, key.as_bytes(), &serialized_data);
            }
            IndexTypeEnum::Key => {
                if let Some(k) = &data.key {
                    let key = key_index_key(&segment_iden.shard_name, k);
                    let index_data = IndexData {
                        segment: segment_iden.segment,
                        offset: data.offset,
                        position,
                        timestamp: 0,
                    };
                    let serialized_data = serialize(&index_data)?;
                    batch.put_cf(&cf, &key, &serialized_data);
                }
            }
            IndexTypeEnum::Tag => {
                if let Some(t) = &data.tag {
                    let index_data = IndexData {
                        segment: segment_iden.segment,
                        offset: data.offset,
                        position,
                        timestamp: 0,
                    };
                    let serialized_data = serialize(&index_data)?;
                    let key = tag_index_key(&segment_iden.shard_name, t, data.offset);
                    batch.put_cf(&cf, key.as_bytes(), &serialized_data);
                }
            }
            IndexTypeEnum::Time => {
                if let Some(t) = data.timestamp {
                    let key = segment_timestamp_index_key(
                        &segment_iden.shard_name,
                        segment_iden.segment,
                        t,
                    );
                    let index_data = IndexData {
                        segment: segment_iden.segment,
                        offset: data.offset,
                        position,
                        timestamp: t,
                    };
                    let serialized_data = serialize(&index_data)?;
                    batch.put_cf(&cf, key.as_bytes(), &serialized_data);
                }
            }
        }
    }

    if batch.is_empty() {
        return Ok(());
    }

    rocksdb_engine_handler.write_batch(batch)?;
    Ok(())
}

pub fn delete_segment_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    let cf = super::get_storage_cf(rocksdb_engine_handler)?;
    let prefix = segment_prefix(&segment_iden.shard_name, segment_iden.segment);
    rocksdb_engine_handler.delete_prefix(cf, &prefix)?;
    Ok(())
}

pub fn delete_shard_index_for_segment(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    segment_seq: u32,
) -> Result<(), StorageEngineError> {
    let cf = super::get_storage_cf(rocksdb_engine_handler)?;
    let mut batch = WriteBatch::default();

    for prefix in [key_index_prefix(shard_name), tag_index_prefix(shard_name)] {
        let mut iter = rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(prefix.as_bytes());
        while iter.valid() {
            let (Some(k), Some(v)) = (iter.key(), iter.value()) else {
                iter.next();
                continue;
            };
            // Compare raw bytes, not via UTF-8: key-index entries embed
            // arbitrary binary record keys that need not be valid UTF-8.
            if !k.starts_with(prefix.as_bytes()) {
                break;
            }
            if let Ok(data) = deserialize::<IndexData>(v) {
                if data.segment == segment_seq {
                    batch.delete_cf(&cf, k);
                }
            }
            iter.next();
        }
    }

    rocksdb_engine_handler.write_batch(batch)?;
    Ok(())
}
