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

pub struct CommitLogOffset {
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}
impl CommitLogOffset {
    pub fn new(
        cache_manager: Arc<StorageCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        CommitLogOffset {
            cache_manager,
            rocksdb_engine_handler,
        }
    }
    pub fn save_latest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        self.save_latest_offset_by_shard(shard, offset)?;
        self.cache_manager.update_latest_offset(shard, offset);
        Ok(())
    }

    pub fn get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard_name) {
            Ok(state.latest_offset)
        } else {
            let (_, latest_offset) = self.recover_shard_data(shard_name)?;
            Ok(latest_offset)
        }
    }

    pub fn save_earliest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        self.save_earliest_offset_by_shard(shard, offset)?;
        self.cache_manager.update_earliest_offset(shard, offset);
        Ok(())
    }

    pub fn get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard_name) {
            Ok(state.earliest_offset)
        } else {
            let (earliest_offset, _) = self.recover_shard_data(shard_name)?;
            Ok(earliest_offset)
        }
    }

    //======== earliest offset ========
    fn save_earliest_offset_by_shard(
        &self,
        shard: &str,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let key = shard_earliest_offset(shard);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            offset,
        )?)
    }

    fn read_earliest_offset_by_shard(
        &self,
        shard: &str,
    ) -> Result<Option<u64>, StorageEngineError> {
        let key = shard_earliest_offset(shard);
        if let Some(res) = engine_get_by_engine::<u64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )? {
            return Ok(Some(res.data));
        }

        Ok(None)
    }

    //======== high watermark offset ========
    fn _save_high_watermark_offset_by_shard(
        &self,
        shard: &str,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let key = shard_high_watermark_offset(shard);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            offset,
        )?)
    }

    fn _read_high_watermark_offset_by_shard(
        &self,
        shard: &str,
    ) -> Result<Option<u64>, StorageEngineError> {
        let key = shard_high_watermark_offset(shard);
        if let Some(res) = engine_get_by_engine::<u64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )? {
            return Ok(Some(res.data));
        }

        Ok(None)
    }

    //======== latest offset ========
    fn save_latest_offset_by_shard(
        &self,
        shard: &str,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let key = shard_latest_offset(shard);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            offset,
        )?)
    }

    fn read_latest_offset_by_shard(&self, shard: &str) -> Result<Option<u64>, StorageEngineError> {
        let key = shard_latest_offset(shard);
        if let Some(res) = engine_get_by_engine::<u64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )? {
            return Ok(Some(res.data));
        }

        Ok(None)
    }

    fn recover_shard_data(&self, shard_name: &str) -> Result<(u64, u64), StorageEngineError> {
        let earliest_offset =
            if let Some(offset) = self.read_earliest_offset_by_shard(shard_name)? {
                offset
            } else {
                return Err(StorageEngineError::CommonErrorStr("".to_string()));
            };

        let latest_offset = if let Some(offset) = self.read_latest_offset_by_shard(shard_name)? {
            offset
        } else {
            return Err(StorageEngineError::CommonErrorStr("".to_string()));
        };

        self.cache_manager.save_offset_state(
            shard_name.to_string(),
            ShardOffsetState {
                earliest_offset,
                latest_offset,
                high_watermark_offset: 0,
            },
        );

        Ok((earliest_offset, latest_offset))
    }
}

#[cfg(test)]
mod tests {}
