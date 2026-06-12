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

pub struct CommitLogOffset {
    pub cache_manager: Arc<StorageCacheManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
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
    // ===== latest offset (LEO) =====
    pub fn save_latest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        self.save_offset(&shard_latest_offset(shard), offset)?;
        self.cache_manager.update_latest_offset(shard, offset);
        Ok(())
    }

    pub fn get_latest_offset(&self, shard: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard) {
            Ok(state.latest_offset)
        } else {
            Ok(self.recover_shard_data(shard)?.latest_offset)
        }
    }

    // ===== earliest offset (log start) =====
    pub fn save_earliest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        self.save_offset(&shard_earliest_offset(shard), offset)?;
        self.cache_manager.update_earliest_offset(shard, offset);
        Ok(())
    }

    pub fn get_earliest_offset(&self, shard: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard) {
            Ok(state.earliest_offset)
        } else {
            Ok(self.recover_shard_data(shard)?.earliest_offset)
        }
    }

    // ===== high watermark（HW） =====
    pub fn save_high_watermark_offset(
        &self,
        shard: &str,
        offset: u64,
    ) -> Result<bool, StorageEngineError> {
        let advanced = self
            .cache_manager
            .update_high_watermark_offset(shard, offset);
        if advanced {
            self.save_offset(&shard_high_watermark_offset(shard), offset)?;
        }
        Ok(advanced)
    }

    pub fn get_high_watermark_offset(&self, shard: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.cache_manager.get_offset_state(shard) {
            Ok(state.high_watermark_offset)
        } else {
            Ok(self
                .load_offset(&shard_high_watermark_offset(shard))?
                .unwrap_or(0))
        }
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
        let missing = |what: &str| {
            StorageEngineError::CommonErrorStr(format!(
                "Failed to recover shard '{shard}': {what} offset not found in storage"
            ))
        };
        let earliest_offset = self
            .load_offset(&shard_earliest_offset(shard))?
            .ok_or_else(|| missing("earliest"))?;
        let latest_offset = self
            .load_offset(&shard_latest_offset(shard))?
            .ok_or_else(|| missing("latest"))?;
        let high_watermark_offset = self
            .load_offset(&shard_high_watermark_offset(shard))?
            .unwrap_or(0)
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

    #[tokio::test]
    async fn test_save_and_read_earliest_offset() {
        let rocksdb_engine = test_rocksdb_instance();
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let offset_manager = CommitLogOffset::new(cache_manager, rocksdb_engine.clone());

        let shard_name = "test_shard";
        let offset = 12345u64;

        offset_manager
            .save_offset(&shard_earliest_offset(shard_name), offset)
            .unwrap();
        let result = offset_manager
            .load_offset(&shard_earliest_offset(shard_name))
            .unwrap();

        assert_eq!(result, Some(offset));
    }

    #[tokio::test]
    async fn test_save_and_read_high_watermark_offset() {
        let rocksdb_engine = test_rocksdb_instance();
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let offset_manager = CommitLogOffset::new(cache_manager, rocksdb_engine.clone());

        let shard_name = "test_shard";
        let offset = 67890u64;

        offset_manager
            .save_offset(&shard_high_watermark_offset(shard_name), offset)
            .unwrap();
        let result = offset_manager
            .load_offset(&shard_high_watermark_offset(shard_name))
            .unwrap();

        assert_eq!(result, Some(offset));
    }

    #[tokio::test]
    async fn test_save_and_read_latest_offset() {
        let rocksdb_engine = test_rocksdb_instance();
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let offset_manager = CommitLogOffset::new(cache_manager, rocksdb_engine.clone());

        let shard_name = "test_shard";
        let offset = 99999u64;

        offset_manager
            .save_offset(&shard_latest_offset(shard_name), offset)
            .unwrap();
        let result = offset_manager
            .load_offset(&shard_latest_offset(shard_name))
            .unwrap();

        assert_eq!(result, Some(offset));
    }
}
