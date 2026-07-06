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

use crate::core::{cache::StorageCacheManager, error::StorageEngineError};
use rocksdb_engine::{
    keys::engine::{shard_earliest_offset, shard_high_watermark_offset, shard_latest_offset},
    rocksdb::RocksDBEngine,
    storage::{
        engine::{engine_get_by_engine, engine_save_by_engine},
        family::DB_COLUMN_FAMILY_STORAGE_ENGINE,
    },
};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ShardOffsetState {
    pub earliest_offset: u64,
    pub high_watermark_offset: u64,
    pub latest_offset: u64,
}

#[derive(Clone)]
pub struct ShardOffset {
    pub cache_manager: Arc<StorageCacheManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShardOffset {
    pub fn new(
        cache_manager: Arc<StorageCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        ShardOffset {
            cache_manager,
            rocksdb_engine_handler,
        }
    }

    pub fn save_latest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        self.save_offset(&shard_latest_offset(shard), offset)?;
        self.cache_manager.update_latest_offset(shard, offset);
        Ok(())
    }

    pub fn get_latest_offset(&self, shard: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard) {
            return Ok(state.latest_offset);
        }
        self.load_offset(&shard_latest_offset(shard))?
            .ok_or_else(|| StorageEngineError::NotOffsetState(shard.to_string()))
    }

    pub fn save_earliest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        self.save_offset(&shard_earliest_offset(shard), offset)?;
        self.cache_manager.update_earliest_offset(shard, offset);
        Ok(())
    }

    pub fn get_earliest_offset(&self, shard: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard) {
            return Ok(state.earliest_offset);
        }
        self.load_offset(&shard_earliest_offset(shard))?
            .ok_or_else(|| StorageEngineError::NotOffsetState(shard.to_string()))
    }

    pub fn save_high_watermark_offset(
        &self,
        shard: &str,
        offset: u64,
    ) -> Result<bool, StorageEngineError> {
        let advanced = self
            .cache_manager
            .update_high_watermark_offset(shard, offset);
        // Save when: initializing (offset=0), HW advanced, or no cache entry yet
        if offset == 0 || advanced || self.cache_manager.get_offset_state(shard).is_none() {
            self.save_offset(&shard_high_watermark_offset(shard), offset)?;
        }
        Ok(advanced)
    }

    pub fn get_high_watermark_offset(&self, shard: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard) {
            return Ok(state.high_watermark_offset);
        }
        Ok(self
            .load_offset(&shard_high_watermark_offset(shard))?
            .unwrap_or(0))
    }

    pub fn get_shard_offsets(&self, shard: &str) -> Result<ShardOffsetState, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard) {
            return Ok(state);
        }
        self.recover_shard_data(shard)
    }

    fn save_offset(&self, key: &str, offset: u64) -> Result<(), StorageEngineError> {
        engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            key,
            offset,
        )?;
        Ok(())
    }

    fn load_offset(&self, key: &str) -> Result<Option<u64>, StorageEngineError> {
        Ok(engine_get_by_engine::<u64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            key,
        )?
        .map(|res| res.data))
    }

    fn recover_shard_data(&self, shard: &str) -> Result<ShardOffsetState, StorageEngineError> {
        let earliest_offset = self
            .load_offset(&shard_earliest_offset(shard))?
            .ok_or_else(|| StorageEngineError::NotOffsetState(shard.to_string()))?;
        let latest_offset = self
            .load_offset(&shard_latest_offset(shard))?
            .ok_or_else(|| StorageEngineError::NotOffsetState(shard.to_string()))?;
        let high_watermark_offset = self
            .load_offset(&shard_high_watermark_offset(shard))?
            .ok_or_else(|| StorageEngineError::NotOffsetState(shard.to_string()))?
            .min(latest_offset);
        let state = ShardOffsetState {
            earliest_offset,
            latest_offset,
            high_watermark_offset,
        };
        self.cache_manager
            .save_offset_state(shard.to_string(), state.clone());
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cache::StorageCacheManager;
    use broker_core::cache::NodeCacheManager;
    use common_config::config::BrokerConfig;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::sync::Arc;

    fn make_offset() -> ShardOffset {
        let rocksdb = test_rocksdb_instance();
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let cache = Arc::new(StorageCacheManager::new(broker_cache));
        ShardOffset::new(cache, rocksdb)
    }

    #[tokio::test]
    async fn save_and_get_latest_offset() {
        let o = make_offset();
        o.save_latest_offset("s1", 42).unwrap();
        assert_eq!(o.get_latest_offset("s1").unwrap(), 42);
    }

    #[tokio::test]
    async fn save_and_get_earliest_offset() {
        let o = make_offset();
        o.save_earliest_offset("s1", 10).unwrap();
        assert_eq!(o.get_earliest_offset("s1").unwrap(), 10);
    }

    #[tokio::test]
    async fn save_and_get_high_watermark_offset() {
        let o = make_offset();
        o.save_high_watermark_offset("s1", 100).unwrap();
        assert_eq!(o.get_high_watermark_offset("s1").unwrap(), 100);
    }

    #[tokio::test]
    async fn recover_from_rocksdb_when_cache_empty() {
        let rocksdb = test_rocksdb_instance();
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let cache = Arc::new(StorageCacheManager::new(broker_cache));
        let o1 = ShardOffset::new(cache.clone(), rocksdb.clone());
        o1.save_latest_offset("s2", 77).unwrap();
        o1.save_earliest_offset("s2", 5).unwrap();

        let o2 = ShardOffset::new(cache, rocksdb);
        assert_eq!(o2.get_latest_offset("s2").unwrap(), 77);
        assert_eq!(o2.get_earliest_offset("s2").unwrap(), 5);
    }
}
