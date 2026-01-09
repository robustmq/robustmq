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
use common_base::utils::serialize::serialize;
use rocksdb::WriteBatch;
use rocksdb_engine::keys::engine::{
    index_key_key, index_position_key, index_tag_key, index_timestamp_key, segment_base,
};
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::{
        engine::engine_list_by_prefix_to_map_by_engine, family::DB_COLUMN_FAMILY_STORAGE_ENGINE,
    },
};
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
    pub key: Option<String>,
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
                    index_position_key(&segment_iden.shard_name, segment_iden.segment, data.offset);
                batch.put_cf(&cf, key.as_bytes(), &serialized_data);
            }
            IndexTypeEnum::Key => {
                if let Some(k) = &data.key {
                    let key = index_key_key(&segment_iden.shard_name, k.clone());
                    let index_data = IndexData {
                        segment: segment_iden.segment,
                        offset: data.offset,
                        position,
                        timestamp: 0,
                    };
                    let serialized_data = serialize(&index_data)?;
                    batch.put_cf(&cf, key.as_bytes(), &serialized_data);
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
                    let key = index_tag_key(&segment_iden.shard_name, t.clone(), data.offset);
                    batch.put_cf(&cf, key.as_bytes(), &serialized_data);
                }
            }
            IndexTypeEnum::Time => {
                if let Some(t) = data.timestamp {
                    let key =
                        index_timestamp_key(&segment_iden.shard_name, segment_iden.segment, t);
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
    let prefix_key_name = segment_base(&segment_iden.shard_name, segment_iden.segment);
    let data = engine_list_by_prefix_to_map_by_engine::<IndexData>(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &prefix_key_name,
    )?;

    if data.is_empty() {
        return Ok(());
    }

    let cf = rocksdb_engine_handler
        .cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE)
        .ok_or_else(|| {
            StorageEngineError::CommonErrorStr(format!(
                "Column family '{}' not found",
                DB_COLUMN_FAMILY_STORAGE_ENGINE
            ))
        })?;

    let mut batch = WriteBatch::default();
    for raw in data.iter() {
        batch.delete_cf(&cf, raw.key().as_bytes());
    }
    rocksdb_engine_handler.write_batch(batch)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_build_segment;
    use crate::filesegment::index::read::{
        get_index_data_by_key, get_index_data_by_offset, get_index_data_by_tag,
        get_index_data_by_timestamp,
    };
    use rocksdb_engine::test::test_rocksdb_instance;

    #[test]
    fn offset_index_save_and_read_test() {
        let rocksdb = test_rocksdb_instance();
        let segment_iden = test_build_segment();

        let mut offset_positions = HashMap::new();
        offset_positions.insert(0, 100);
        offset_positions.insert(10000, 50000);
        offset_positions.insert(20000, 150000);

        let index_data = vec![
            BuildIndexRaw {
                index_type: IndexTypeEnum::Offset,
                offset: 0,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Offset,
                offset: 10000,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Offset,
                offset: 20000,
                ..Default::default()
            },
        ];

        save_index(&rocksdb, &segment_iden, &index_data, &offset_positions).unwrap();

        let result = get_index_data_by_offset(&rocksdb, &segment_iden, 0).unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 0);
        assert_eq!(data.position, 100);

        let result = get_index_data_by_offset(&rocksdb, &segment_iden, 15000).unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 10000);
        assert_eq!(data.position, 50000);

        let result = get_index_data_by_offset(&rocksdb, &segment_iden, 25000).unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 20000);
        assert_eq!(data.position, 150000);
    }

    #[test]
    fn key_index_save_and_read_test() {
        let rocksdb = test_rocksdb_instance();
        let mut segment_iden = test_build_segment();
        segment_iden.segment = 20;

        let mut offset_positions = HashMap::new();
        offset_positions.insert(100, 1000);
        offset_positions.insert(200, 2000);
        offset_positions.insert(300, 3000);

        let index_data = vec![
            BuildIndexRaw {
                index_type: IndexTypeEnum::Key,
                key: Some("user-123".to_string()),
                offset: 100,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Key,
                key: Some("order-456".to_string()),
                offset: 200,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Key,
                key: Some("product-789".to_string()),
                offset: 300,
                ..Default::default()
            },
        ];

        save_index(&rocksdb, &segment_iden, &index_data, &offset_positions).unwrap();

        let result =
            get_index_data_by_key(&rocksdb, &segment_iden.shard_name, "user-123".to_string())
                .unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 100);
        assert_eq!(data.position, 1000);

        let result =
            get_index_data_by_key(&rocksdb, &segment_iden.shard_name, "order-456".to_string())
                .unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 200);
        assert_eq!(data.position, 2000);

        let result =
            get_index_data_by_key(&rocksdb, &segment_iden.shard_name, "not-exist".to_string())
                .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn tag_index_save_and_read_test() {
        let rocksdb = test_rocksdb_instance();
        let mut segment_iden = test_build_segment();
        segment_iden.segment = 30;

        let mut offset_positions = HashMap::new();
        offset_positions.insert(100, 1000);
        offset_positions.insert(200, 2000);
        offset_positions.insert(300, 3000);
        offset_positions.insert(400, 4000);

        let index_data = vec![
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("urgent".to_string()),
                offset: 100,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("urgent".to_string()),
                offset: 200,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("normal".to_string()),
                offset: 300,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("urgent".to_string()),
                offset: 400,
                ..Default::default()
            },
        ];

        save_index(&rocksdb, &segment_iden, &index_data, &offset_positions).unwrap();

        let results =
            get_index_data_by_tag(&rocksdb, &segment_iden.shard_name, Some(0), "urgent", 10)
                .unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].offset, 100);
        assert_eq!(results[0].position, 1000);
        assert_eq!(results[1].offset, 200);
        assert_eq!(results[1].position, 2000);
        assert_eq!(results[2].offset, 400);
        assert_eq!(results[2].position, 4000);

        let results =
            get_index_data_by_tag(&rocksdb, &segment_iden.shard_name, Some(150), "urgent", 10)
                .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].offset, 200);
        assert_eq!(results[1].offset, 400);

        let results =
            get_index_data_by_tag(&rocksdb, &segment_iden.shard_name, Some(0), "normal", 10)
                .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].offset, 300);
    }

    #[test]
    fn timestamp_index_save_and_read_test() {
        let rocksdb = test_rocksdb_instance();
        let mut segment_iden = test_build_segment();
        segment_iden.segment = 40;

        let mut offset_positions = HashMap::new();
        offset_positions.insert(0, 100);
        offset_positions.insert(10000, 50000);
        offset_positions.insert(20000, 150000);

        let index_data = vec![
            BuildIndexRaw {
                index_type: IndexTypeEnum::Time,
                timestamp: Some(1000),
                offset: 0,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Time,
                timestamp: Some(2000),
                offset: 10000,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Time,
                timestamp: Some(3000),
                offset: 20000,
                ..Default::default()
            },
        ];

        save_index(&rocksdb, &segment_iden, &index_data, &offset_positions).unwrap();

        let result = get_index_data_by_timestamp(&rocksdb, &segment_iden, 1000).unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 0);
        assert_eq!(data.position, 100);
        assert_eq!(data.timestamp, 1000);

        let result = get_index_data_by_timestamp(&rocksdb, &segment_iden, 1500).unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 0);
        assert_eq!(data.position, 100);
        assert_eq!(data.timestamp, 1000);

        let result = get_index_data_by_timestamp(&rocksdb, &segment_iden, 2500).unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 10000);
        assert_eq!(data.position, 50000);
        assert_eq!(data.timestamp, 2000);

        let result = get_index_data_by_timestamp(&rocksdb, &segment_iden, 3500).unwrap();
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.offset, 20000);
        assert_eq!(data.position, 150000);
        assert_eq!(data.timestamp, 3000);
    }
}
