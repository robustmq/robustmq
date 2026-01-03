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
    core::{cache::StorageCacheManager, error::StorageEngineError},
    segment::{index::segment::SegmentIndexManager, SegmentIdentity},
};
use metadata_struct::storage::shard::EngineStorageType;
use rocksdb_engine::{
    keys::engine::{shard_earliest_offset, shard_high_watermark_offset, shard_latest_offset},
    rocksdb::RocksDBEngine,
    storage::{
        engine::{engine_get_by_engine, engine_save_by_engine},
        family::DB_COLUMN_FAMILY_STORAGE_ENGINE,
    },
};
use std::sync::Arc;

//======== earliest offset ========
pub fn save_earliest_offset_by_shard(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    offset: u64,
) -> Result<(), StorageEngineError> {
    let key = shard_earliest_offset(shard);
    Ok(engine_save_by_engine(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &key,
        offset,
    )?)
}

pub fn read_earliest_offset_by_shard(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
) -> Result<Option<u64>, StorageEngineError> {
    let key = shard_earliest_offset(shard);
    if let Some(res) = engine_get_by_engine::<u64>(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &key,
    )? {
        return Ok(Some(res.data));
    }

    Ok(None)
}

//======== high watermark offset ========
pub fn save_high_watermark_offset_by_shard(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    offset: u64,
) -> Result<(), StorageEngineError> {
    let key = shard_high_watermark_offset(shard);
    Ok(engine_save_by_engine(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &key,
        offset,
    )?)
}

pub fn read_high_watermark_offset_by_shard(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
) -> Result<Option<u64>, StorageEngineError> {
    let key = shard_high_watermark_offset(shard);
    if let Some(res) = engine_get_by_engine::<u64>(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &key,
    )? {
        return Ok(Some(res.data));
    }

    Ok(None)
}

//======== latest offset ========
pub fn save_latest_offset_by_shard(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    offset: u64,
) -> Result<(), StorageEngineError> {
    let key = shard_latest_offset(shard);
    Ok(engine_save_by_engine(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &key,
        offset,
    )?)
}

pub fn read_latest_offset_by_shard(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
) -> Result<Option<u64>, StorageEngineError> {
    let key = shard_latest_offset(shard);
    if let Some(res) = engine_get_by_engine::<u64>(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &key,
    )? {
        return Ok(Some(res.data));
    }

    Ok(None)
}

// segment earliest/high_watermark/latest info
pub fn get_earliest_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
) -> Result<u64, StorageEngineError> {
    let shard = cache_manager
        .shards
        .get(shard_name)
        .ok_or_else(|| StorageEngineError::ShardNotExist(shard_name.to_string()))?;

    match shard.get_engine_type()? {
        EngineStorageType::Memory | EngineStorageType::RocksDB => {
            if let Some(offset) = read_earliest_offset_by_shard(rocksdb_engine_handler, shard_name)?
            {
                Ok(offset)
            } else {
                Ok(0)
            }
        }

        EngineStorageType::Segment => {
            let segment_iden = SegmentIdentity::new(shard_name, shard.start_segment_seq);
            let segment_meta = cache_manager
                .get_segment_meta(&segment_iden)
                .ok_or_else(|| StorageEngineError::SegmentNotExist(segment_iden.name()))?;

            if segment_meta.start_offset < 0 {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Invalid start offset {} for shard {} segment {}",
                    segment_meta.start_offset, shard_name, shard.start_segment_seq
                )));
            }
            Ok(segment_meta.start_offset as u64)
        }
    }
}

pub fn get_latest_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
) -> Result<u64, StorageEngineError> {
    let shard = cache_manager
        .shards
        .get(shard_name)
        .ok_or_else(|| StorageEngineError::ShardNotExist(shard_name.to_string()))?;

    if let Some(offset) = read_latest_offset_by_shard(rocksdb_engine_handler, shard_name)? {
        return Ok(offset);
    }

    match shard.get_engine_type()? {
        EngineStorageType::Memory | EngineStorageType::RocksDB => Ok(0),

        EngineStorageType::Segment => {
            let segment_index_manager = SegmentIndexManager::new(rocksdb_engine_handler.clone());
            let segment_iden = SegmentIdentity::new(shard_name, shard.active_segment_seq);
            let start_offset = segment_index_manager.get_start_offset(&segment_iden)?;
            if start_offset < 0 {
                return Err(StorageEngineError::NoOffsetInformation(
                    shard_name.to_string(),
                ));
            }

            save_latest_offset_by_shard(rocksdb_engine_handler, shard_name, start_offset as u64)?;
            Ok(start_offset as u64)
        }
    }
}

pub fn get_high_water_offset(
    _rocksdb_engine_handler: &Arc<RocksDBEngine>,
    _shard: &str,
) -> Result<u64, StorageEngineError> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use broker_core::cache::BrokerCacheManager;
    use common_base::tools::unique_id;
    use common_config::config::BrokerConfig;
    use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
    use rocksdb_engine::test::test_rocksdb_instance;

    #[test]
    fn test_earliest_offset_save_read() {
        let db = Arc::new(test_rocksdb_instance());
        let shard_name = unique_id();
        let offset = 12345u64;

        save_earliest_offset_by_shard(&db, &shard_name, offset).unwrap();
        let result = read_earliest_offset_by_shard(&db, &shard_name).unwrap();

        assert_eq!(result, Some(offset));
    }

    #[test]
    fn test_high_watermark_offset_save_read() {
        let db = Arc::new(test_rocksdb_instance());
        let shard_name = unique_id();
        let offset = 67890u64;

        save_high_watermark_offset_by_shard(&db, &shard_name, offset).unwrap();
        let result = read_high_watermark_offset_by_shard(&db, &shard_name).unwrap();

        assert_eq!(result, Some(offset));
    }

    #[test]
    fn test_latest_offset_save_read() {
        let db = Arc::new(test_rocksdb_instance());
        let shard_name = unique_id();
        let offset = 99999u64;

        save_latest_offset_by_shard(&db, &shard_name, offset).unwrap();
        let result = read_latest_offset_by_shard(&db, &shard_name).unwrap();

        assert_eq!(result, Some(offset));
    }

    #[test]
    fn test_get_latest_offset() {
        let db = Arc::new(test_rocksdb_instance());
        let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let shard_name = unique_id();
        let offset = 88888u64;

        let shard = EngineShard {
            shard_name: shard_name.clone(),
            config: EngineShardConfig {
                engine_storage_type: Some(EngineStorageType::Memory),
                ..Default::default()
            },
            ..Default::default()
        };
        cache_manager.shards.insert(shard_name.clone(), shard);

        save_latest_offset_by_shard(&db, &shard_name, offset).unwrap();
        let result = get_latest_offset(&db, &cache_manager, &shard_name).unwrap();

        assert_eq!(result, offset);
    }

    #[test]
    fn test_get_latest_offset_default() {
        let db = Arc::new(test_rocksdb_instance());
        let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let shard_name = unique_id();

        let shard = EngineShard {
            shard_name: shard_name.clone(),
            config: EngineShardConfig {
                engine_storage_type: Some(EngineStorageType::Memory),
                ..Default::default()
            },
            ..Default::default()
        };
        cache_manager.shards.insert(shard_name.clone(), shard);

        let result = get_latest_offset(&db, &cache_manager, &shard_name).unwrap();

        assert_eq!(result, 0);
    }
}
