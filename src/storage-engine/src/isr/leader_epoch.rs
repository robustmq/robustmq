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
use rocksdb_engine::keys::engine::{leader_epoch_entry_key, leader_epoch_prefix};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::engine::{
    engine_delete_prefix_by_engine, engine_list_by_prefix_by_engine, engine_save_by_engine,
};
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_STORAGE_ENGINE;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaderEpochEntry {
    pub epoch: u32,
    pub start_offset: u64,
}

pub struct LeaderEpochCache {
    shard: String,
    segment_seq: u32,
    entries: Vec<LeaderEpochEntry>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl LeaderEpochCache {
    pub fn load(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        shard: &str,
        segment_seq: u32,
    ) -> Result<Self, StorageEngineError> {
        let prefix = leader_epoch_prefix(shard, segment_seq);
        let rows = engine_list_by_prefix_by_engine::<LeaderEpochEntry>(
            &rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &prefix,
        )?;

        let entries = rows.into_iter().map(|w| w.data).collect();
        Ok(LeaderEpochCache {
            shard: shard.to_string(),
            segment_seq,
            entries,
            rocksdb_engine_handler,
        })
    }

    pub fn assign(&mut self, epoch: u32, start_offset: u64) -> Result<(), StorageEngineError> {
        if let Some(last) = self.entries.last() {
            if epoch <= last.epoch {
                return Ok(());
            }
        }
        let entry = LeaderEpochEntry {
            epoch,
            start_offset,
        };
        self.persist(&entry)?;
        self.entries.push(entry);
        Ok(())
    }

    pub fn latest_epoch(&self) -> u32 {
        self.entries.last().map(|e| e.epoch).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn end_offset_for(&self, my_epoch: u32) -> Option<u64> {
        self.entries
            .iter()
            .find(|e| e.epoch > my_epoch)
            .map(|e| e.start_offset)
    }

    pub fn truncate_from_end(&mut self, end_offset: u64) -> Result<(), StorageEngineError> {
        let removed: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.start_offset > end_offset)
            .copied()
            .collect();
        for e in &removed {
            self.delete_entry(e.epoch)?;
        }
        self.entries.retain(|e| e.start_offset <= end_offset);
        Ok(())
    }

    pub fn truncate_from_end_by_epoch(
        &mut self,
        target_epoch: u32,
    ) -> Result<(), StorageEngineError> {
        let removed: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.epoch > target_epoch)
            .copied()
            .collect();
        for e in &removed {
            self.delete_entry(e.epoch)?;
        }
        self.entries.retain(|e| e.epoch <= target_epoch);
        Ok(())
    }

    pub fn truncate_from_start(&mut self, start_offset: u64) -> Result<(), StorageEngineError> {
        let keep_from = self
            .entries
            .iter()
            .rposition(|e| e.start_offset <= start_offset)
            .unwrap_or(0);
        let removed: Vec<_> = self.entries[..keep_from].to_vec();
        for e in &removed {
            self.delete_entry(e.epoch)?;
        }
        self.entries.drain(..keep_from);
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), StorageEngineError> {
        let prefix = leader_epoch_prefix(&self.shard, self.segment_seq);
        engine_delete_prefix_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &prefix,
        )?;
        self.entries.clear();
        Ok(())
    }

    fn persist(&self, entry: &LeaderEpochEntry) -> Result<(), StorageEngineError> {
        let key = leader_epoch_entry_key(&self.shard, self.segment_seq, entry.epoch);
        engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            *entry,
        )?;
        Ok(())
    }

    fn delete_entry(&self, epoch: u32) -> Result<(), StorageEngineError> {
        let key = leader_epoch_entry_key(&self.shard, self.segment_seq, epoch);
        rocksdb_engine::storage::engine::engine_delete_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::LeaderEpochCache;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn cache() -> LeaderEpochCache {
        LeaderEpochCache::load(test_rocksdb_instance(), "s", 0).unwrap()
    }

    #[test]
    fn assign_query_and_reload() {
        let db = test_rocksdb_instance();
        let mut c = LeaderEpochCache::load(db.clone(), "s", 0).unwrap();
        assert_eq!(c.latest_epoch(), 0);
        assert!(c.is_empty());

        c.assign(1, 0).unwrap();
        c.assign(3, 10).unwrap();
        c.assign(3, 99).unwrap();
        c.assign(2, 99).unwrap();
        assert_eq!(c.latest_epoch(), 3);
        assert_eq!(c.end_offset_for(1), Some(10));
        assert_eq!(c.end_offset_for(2), Some(10));
        assert_eq!(c.end_offset_for(3), None);

        let reloaded = LeaderEpochCache::load(db, "s", 0).unwrap();
        assert_eq!(reloaded.latest_epoch(), 3);
        assert_eq!(reloaded.end_offset_for(1), Some(10));
    }

    #[test]
    fn truncate() {
        let mut c = cache();
        c.assign(1, 0).unwrap();
        c.assign(2, 5).unwrap();
        c.assign(3, 10).unwrap();

        c.truncate_from_end(7).unwrap();
        assert_eq!(c.latest_epoch(), 2);

        c.assign(3, 10).unwrap();
        c.truncate_from_end_by_epoch(1).unwrap();
        assert_eq!(c.latest_epoch(), 1);
        assert_eq!(c.end_offset_for(1), None);

        c.assign(2, 5).unwrap();
        c.assign(3, 10).unwrap();
        c.truncate_from_start(7).unwrap();
        assert_eq!(c.latest_epoch(), 3);
        assert_eq!(c.end_offset_for(1), Some(5));
    }

    #[test]
    fn clear_empties() {
        let mut c = cache();
        c.assign(1, 0).unwrap();
        c.assign(2, 5).unwrap();
        c.clear().unwrap();
        assert!(c.is_empty());
        assert_eq!(c.latest_epoch(), 0);
    }
}
