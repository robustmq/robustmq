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

use crate::core::{cache::StorageCacheManager, error::StorageEngineError, shard::ShardOffsetState};
use rocksdb_engine::{
    keys::engine::{shard_earliest_offset, shard_high_watermark_offset, shard_latest_offset},
    rocksdb::RocksDBEngine,
    storage::{
        engine::{engine_get_by_engine, engine_save_by_engine},
        family::DB_COLUMN_FAMILY_STORAGE_ENGINE,
    },
};
use std::sync::Arc;

pub fn save_latest_offset(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    offset: u64,
) -> Result<(), StorageEngineError> {
    save_latest_offset_by_shard(rocksdb_engine_handler, shard, offset)?;
    cache_manager.update_latest_offset(shard, offset);
    Ok(())
}

pub fn get_latest_offset(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
) -> Result<u64, StorageEngineError> {
    if let Some(state) = cache_manager.get_offset_state(shard_name) {
        Ok(state.latest_offset)
    } else {
        let (_, latest_offset) =
            recover_shard_data(cache_manager, rocksdb_engine_handler, shard_name)?;
        Ok(latest_offset)
    }
}

pub fn save_earliest_offset(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    offset: u64,
) -> Result<(), StorageEngineError> {
    save_earliest_offset_by_shard(rocksdb_engine_handler, shard, offset)?;
    cache_manager.update_earliest_offset(shard, offset);
    Ok(())
}

pub fn get_earliest_offset(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
) -> Result<u64, StorageEngineError> {
    if let Some(state) = cache_manager.get_offset_state(shard_name) {
        Ok(state.earliest_offset)
    } else {
        let (earliest_offset, _) =
            recover_shard_data(cache_manager, rocksdb_engine_handler, shard_name)?;
        Ok(earliest_offset)
    }
}

//======== earliest offset ========
fn save_earliest_offset_by_shard(
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

fn read_earliest_offset_by_shard(
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
fn save_high_watermark_offset_by_shard(
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

fn read_high_watermark_offset_by_shard(
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
fn save_latest_offset_by_shard(
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

fn read_latest_offset_by_shard(
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

fn recover_shard_data(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
) -> Result<(u64, u64), StorageEngineError> {
    let earliest_offset =
        if let Some(offset) = read_earliest_offset_by_shard(rocksdb_engine_handler, shard_name)? {
            offset
        } else {
            return Err(StorageEngineError::CommonErrorStr("".to_string()));
        };

    let latest_offset =
        if let Some(offset) = read_latest_offset_by_shard(rocksdb_engine_handler, shard_name)? {
            offset
        } else {
            return Err(StorageEngineError::CommonErrorStr("".to_string()));
        };

    cache_manager.save_offset_state(
        shard_name.to_string(),
        ShardOffsetState {
            earliest_offset,
            latest_offset,
            high_watermark_offset: 0,
        },
    );

    Ok((earliest_offset, latest_offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::unique_id;
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
}
