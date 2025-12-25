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

use crate::{
    core::error::StorageEngineError,
    segment::{
        keys::{
            key_segment, offset_segment_position, segment_index_prefix, tag_segment,
            timestamp_segment_time,
        },
        SegmentIdentity,
    },
};
use common_base::utils::serialize::serialize;
use rocksdb::WriteBatch;
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::{
        engine::{engine_delete_by_engine, engine_list_by_prefix_to_map_by_engine},
        family::DB_COLUMN_FAMILY_STORAGE_ENGINE,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IndexData {
    pub offset: u64,
    pub timestamp: u64,
    pub position: u64,
}

pub fn delete_segment_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    let prefix_key_name = segment_index_prefix(segment_iden);
    let data = engine_list_by_prefix_to_map_by_engine::<IndexData>(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &prefix_key_name,
    )?;
    for raw in data.iter() {
        engine_delete_by_engine(
            rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            raw.key(),
        )?;
    }
    Ok(())
}

#[derive(Default)]
pub enum IndexTypeEnum {
    #[default]
    Offset,
    Tag,
    Key,
    Time,
}

#[derive(Default)]
pub struct BuildIndexRaw {
    pub index_type: IndexTypeEnum,
    pub segment_iden: SegmentIdentity,
    pub index_data: IndexData,
    pub key: Option<String>,
    pub tag: Option<String>,
    pub timestamp: Option<u64>,
}

pub fn save_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    index_data: &[BuildIndexRaw],
) -> Result<(), StorageEngineError> {
    let cf = rocksdb_engine_handler
        .cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE)
        .ok_or_else(|| {
            StorageEngineError::CommonErrorStr(format!(
                "Column family '{}' not found",
                DB_COLUMN_FAMILY_STORAGE_ENGINE
            ))
        })?;

    let mut batch = WriteBatch::default();
    for data in index_data.iter() {
        let serialized_data = serialize(&data.index_data)?;
        match data.index_type {
            IndexTypeEnum::Offset => {
                let key = offset_segment_position(&data.segment_iden, data.index_data.offset);
                batch.put_cf(&cf, key.as_bytes(), &serialized_data);
            }
            IndexTypeEnum::Key => {
                if let Some(k) = data.key.clone() {
                    let key = key_segment(segment_iden, k);
                    batch.put_cf(&cf, key.as_bytes(), &serialized_data);
                }
            }
            IndexTypeEnum::Tag => {
                if let Some(t) = data.tag.clone() {
                    let key = tag_segment(segment_iden, t, data.index_data.offset);
                    batch.put_cf(&cf, key.as_bytes(), &serialized_data);
                }
            }
            IndexTypeEnum::Time => {
                if let Some(t) = data.timestamp {
                    let key = timestamp_segment_time(segment_iden, t);
                    batch.put_cf(&cf, key.as_bytes(), &serialized_data);
                }
            }
        }
    }

    rocksdb_engine_handler.write_batch(batch)?;
    Ok(())
}
