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

//! `ShardOffsetManager` — unified shard-level offset query interface.
//!
//! Both `CommitLogOffset` (Memory / RocksDB) and `SegmentOffset` (EngineSegment)
//! implement this trait so that upper-layer code can query offsets without
//! branching on `storage_type`.

use crate::core::error::StorageEngineError;
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use std::sync::Arc;

/// Shard-level offset query interface.
///
/// Engines that track offset metadata (latest committed, earliest retained,
/// timestamp-to-offset lookup) implement this trait to expose a uniform API.
///
/// The `get_offset_by_timestamp` method has a default implementation that
/// falls back to the strategy-selected bound (`Earliest` → `get_earliest_offset`,
/// `Latest` → `get_latest_offset`). Engines with per-record timestamp indices
/// (e.g. `SegmentOffset`) should override this to return a precise answer.
pub trait ShardOffsetManager: Send + Sync {
    /// Returns the log-end offset (next-write position) for `shard_name`.
    fn get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError>;

    /// Returns the earliest retained offset for `shard_name`.
    fn get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError>;

    /// Returns the offset nearest to `timestamp` for `shard_name`.
    ///
    /// The default implementation ignores `_timestamp` and returns the
    /// strategy-selected bound, which is correct for engines that do not
    /// maintain per-record timestamp indices.
    fn get_offset_by_timestamp(
        &self,
        shard_name: &str,
        _timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, StorageEngineError> {
        match strategy {
            AdapterOffsetStrategy::Earliest => self.get_earliest_offset(shard_name),
            AdapterOffsetStrategy::Latest => self.get_latest_offset(shard_name),
        }
    }
}

impl<T: ShardOffsetManager> ShardOffsetManager for Arc<T> {
    fn get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        self.as_ref().get_latest_offset(shard_name)
    }

    fn get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        self.as_ref().get_earliest_offset(shard_name)
    }

    fn get_offset_by_timestamp(
        &self,
        shard_name: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, StorageEngineError> {
        self.as_ref()
            .get_offset_by_timestamp(shard_name, timestamp, strategy)
    }
}

#[cfg(test)]
mod tests {
    use super::ShardOffsetManager;
    use crate::core::error::StorageEngineError;
    use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;

    struct StubOffsetManager {
        earliest: u64,
        latest: u64,
    }

    impl ShardOffsetManager for StubOffsetManager {
        fn get_latest_offset(&self, _shard: &str) -> Result<u64, StorageEngineError> {
            Ok(self.latest)
        }
        fn get_earliest_offset(&self, _shard: &str) -> Result<u64, StorageEngineError> {
            Ok(self.earliest)
        }
    }

    #[test]
    fn default_timestamp_fallback_earliest() {
        let mgr = StubOffsetManager {
            earliest: 10,
            latest: 100,
        };
        let result = mgr
            .get_offset_by_timestamp("s", 999, AdapterOffsetStrategy::Earliest)
            .unwrap();
        assert_eq!(result, 10);
    }

    #[test]
    fn default_timestamp_fallback_latest() {
        let mgr = StubOffsetManager {
            earliest: 10,
            latest: 100,
        };
        let result = mgr
            .get_offset_by_timestamp("s", 999, AdapterOffsetStrategy::Latest)
            .unwrap();
        assert_eq!(result, 100);
    }

    #[test]
    fn commit_log_offset_implements_trait() {
        use crate::commitlog::offset::CommitLogOffset;
        use crate::core::test_tool::{test_build_memory_engine, test_init_conf};
        use std::sync::Arc;

        test_init_conf();
        let engine = Arc::new(test_build_memory_engine());
        let cm = engine.cache_manager.clone();

        // CommitLogOffset must be usable as Box<dyn ShardOffsetManager>.
        let clo = CommitLogOffset::new(cm, rocksdb_engine::test::test_rocksdb_instance());
        let _boxed: Box<dyn ShardOffsetManager> = Box::new(clo);
    }

    #[tokio::test]
    async fn segment_offset_implements_trait() {
        use crate::core::test_tool::test_init_segment;
        use crate::filesegment::offset::SegmentOffset;
        use common_config::storage::StorageType;

        let (_, cache_manager, _, rocksdb) = test_init_segment(StorageType::EngineSegment).await;
        let so = SegmentOffset::new(rocksdb, cache_manager);
        let _boxed: Box<dyn ShardOffsetManager> = Box::new(so);
    }
}
