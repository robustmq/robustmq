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

use crate::raft::store::keys::{
    key_committed, key_last_purged_log_id, key_raft_log, key_vote, raft_log_key_to_id,
};
use crate::raft::type_config::{StorageResult, TypeConfig};
use bincode::{deserialize, serialize};
use openraft::storage::{IOFlushed, RaftLogStorage};
use openraft::{Entry, LogId, LogState, OptionalSend, RaftLogReader, StorageError, Vote};
use rocksdb::{BoundColumnFamily, Direction, IteratorMode, WriteBatch, DB};
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_META_RAFT;
use std::fmt::Debug;
use std::ops::RangeBounds;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LogStore {
    pub machine: String,
    pub db: Arc<DB>,
}

impl LogStore {
    #[inline]
    fn store(&self) -> Arc<BoundColumnFamily<'_>> {
        self.db.cf_handle(DB_COLUMN_FAMILY_META_RAFT).unwrap()
    }

    fn get_last_purged_(&self) -> StorageResult<Option<LogId<TypeConfig>>> {
        Ok(self
            .db
            .get_cf(&self.store(), key_last_purged_log_id(&self.machine))
            .map_err(|e| StorageError::read(&e))?
            .and_then(|v| deserialize(&v).ok()))
    }

    fn set_last_purged_(&self, log_id: LogId<TypeConfig>) -> StorageResult<()> {
        let data = serialize(&log_id).map_err(|e| StorageError::write(&e))?;
        self.db
            .put_cf(&self.store(), key_last_purged_log_id(&self.machine), data)
            .map_err(|e| StorageError::write(&e))
    }

    fn set_committed_(&self, committed: &Option<LogId<TypeConfig>>) -> StorageResult<()> {
        match committed {
            Some(log_id) => {
                let data = serialize(log_id).map_err(|e| StorageError::write(&e))?;
                self.db
                    .put_cf(&self.store(), key_committed(&self.machine), data)
                    .map_err(|e| StorageError::write(&e))
            }
            None => self
                .db
                .delete_cf(&self.store(), key_committed(&self.machine))
                .map_err(|e| StorageError::write(&e)),
        }
    }

    fn get_committed_(&self) -> StorageResult<Option<LogId<TypeConfig>>> {
        Ok(self
            .db
            .get_cf(&self.store(), key_committed(&self.machine))
            .map_err(|e| StorageError::read(&e))?
            .and_then(|v| deserialize(&v).ok()))
    }

    fn set_vote_(&self, vote: &Vote<TypeConfig>) -> StorageResult<()> {
        let data = serialize(vote).map_err(|e| StorageError::write_vote(&e))?;
        self.db
            .put_cf(&self.store(), key_vote(&self.machine), data)
            .map_err(|e| StorageError::write_vote(&e))
    }

    fn get_vote_(&self) -> StorageResult<Option<Vote<TypeConfig>>> {
        Ok(self
            .db
            .get_cf(&self.store(), key_vote(&self.machine))
            .map_err(|e| StorageError::write_vote(&e))?
            .and_then(|v| deserialize(&v).ok()))
    }
}

impl RaftLogReader<TypeConfig> for LogStore {
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> StorageResult<Vec<Entry<TypeConfig>>> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(x) => key_raft_log(&self.machine, *x),
            std::ops::Bound::Excluded(x) => key_raft_log(&self.machine, *x + 1),
            std::ops::Bound::Unbounded => key_raft_log(&self.machine, 0),
        };

        let mut entries = Vec::new();

        for item in self.db.iterator_cf(
            &self.store(),
            IteratorMode::From(&start, Direction::Forward),
        ) {
            let (key, val) = item.map_err(|e| StorageError::read_logs(&e))?;

            let id = match raft_log_key_to_id(&self.machine, &key) {
                Ok(id) => id,
                Err(_) => continue,
            };

            if !range.contains(&id) {
                break;
            }

            let entry: Entry<TypeConfig> =
                deserialize(&val).map_err(|e| StorageError::read_logs(&e))?;

            entries.push(entry);
        }

        Ok(entries)
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<TypeConfig>>, StorageError<TypeConfig>> {
        self.get_vote_()
    }
}

impl RaftLogStorage<TypeConfig> for LogStore {
    type LogReader = Self;

    async fn get_log_state(&mut self) -> StorageResult<LogState<TypeConfig>> {
        // Get the last log entry for this specific machine
        // Need to iterate backwards from the machine's max key and find the first valid entry
        let start_key = key_raft_log(&self.machine, u64::MAX);

        let last = self
            .db
            .iterator_cf(
                &self.store(),
                IteratorMode::From(&start_key, Direction::Reverse),
            )
            .find_map(|res| {
                let (key, val) = res.ok()?;
                // Verify the key belongs to this machine
                let _ = raft_log_key_to_id(&self.machine, &key).ok()?;
                deserialize::<Entry<TypeConfig>>(&val)
                    .ok()
                    .map(|e| e.log_id)
            });

        let last_purged_log_id = self.get_last_purged_()?;
        let last_log_id = last.or(last_purged_log_id);

        Ok(LogState {
            last_purged_log_id,
            last_log_id,
        })
    }

    async fn save_committed(
        &mut self,
        committed: Option<LogId<TypeConfig>>,
    ) -> Result<(), StorageError<TypeConfig>> {
        self.set_committed_(&committed)
    }

    async fn read_committed(
        &mut self,
    ) -> Result<Option<LogId<TypeConfig>>, StorageError<TypeConfig>> {
        self.get_committed_()
    }

    async fn save_vote(&mut self, vote: &Vote<TypeConfig>) -> Result<(), StorageError<TypeConfig>> {
        self.set_vote_(vote)
    }

    async fn append<I>(&mut self, entries: I, callback: IOFlushed<TypeConfig>) -> StorageResult<()>
    where
        I: IntoIterator<Item = Entry<TypeConfig>> + Send,
        I::IntoIter: Send,
    {
        let mut batch = WriteBatch::default();
        let mut entry_count = 0;

        for entry in entries {
            let id = key_raft_log(&self.machine, entry.log_id.index);
            let serialized = serialize(&entry).map_err(|e| StorageError::write_logs(&e))?;

            batch.put_cf(&self.store(), id, serialized);
            entry_count += 1;
        }

        if entry_count > 0 {
            self.db
                .write(batch)
                .map_err(|e| StorageError::write_logs(&e))?;
        }

        callback.io_completed(Ok(()));
        Ok(())
    }

    async fn truncate(&mut self, log_id: LogId<TypeConfig>) -> StorageResult<()> {
        let from = key_raft_log(&self.machine, log_id.index);
        let to = key_raft_log(&self.machine, u64::MAX);

        self.db
            .delete_range_cf(&self.store(), &from, &to)
            .map_err(|e| StorageError::write_logs(&e))
    }

    async fn purge(&mut self, log_id: LogId<TypeConfig>) -> Result<(), StorageError<TypeConfig>> {
        self.set_last_purged_(log_id)?;

        let from = key_raft_log(&self.machine, 0);
        let to = key_raft_log(&self.machine, log_id.index + 1);

        self.db
            .delete_range_cf(&self.store(), &from, &to)
            .map_err(|e| StorageError::write_logs(&e))
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raft::route::data::{StorageData, StorageDataType};
    use bytes::Bytes;
    use openraft::vote::leader_id_adv::LeaderId;
    use openraft::LogId;
    use rocksdb::{Options, DB};
    use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_META_RAFT;
    use std::sync::Arc;

    fn create_test_log_store() -> LogStore {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = DB::open_cf(&opts, temp_dir.path(), vec![DB_COLUMN_FAMILY_META_RAFT]).unwrap();

        LogStore {
            machine: "test_machine".to_string(),
            db: Arc::new(db),
        }
    }

    fn create_log_id(term: u64, node_id: u64, index: u64) -> LogId<TypeConfig> {
        LogId {
            leader_id: LeaderId { term, node_id },
            index,
        }
    }

    fn create_entry(term: u64, node_id: u64, index: u64) -> Entry<TypeConfig> {
        Entry {
            log_id: create_log_id(term, node_id, index),
            payload: openraft::EntryPayload::Normal(StorageData::new(
                StorageDataType::KvSet,
                Bytes::from(format!("data_{}", index)),
            )),
        }
    }

    // Helper for testing: simplified append without callback complexity
    async fn append_entries(
        log_store: &mut LogStore,
        entries: Vec<Entry<TypeConfig>>,
    ) -> StorageResult<()> {
        let mut batch = WriteBatch::default();
        for entry in entries {
            let id = key_raft_log(&log_store.machine, entry.log_id.index);
            let serialized = serialize(&entry).map_err(|e| StorageError::write_logs(&e))?;
            batch.put_cf(&log_store.store(), id, serialized);
        }
        log_store
            .db
            .write(batch)
            .map_err(|e| StorageError::write_logs(&e))
    }

    #[tokio::test]
    async fn test_raft_log_operations() {
        let mut log_store = create_test_log_store();

        let entries: Vec<_> = (1..=10).map(|i| create_entry(1, 1, i)).collect();
        append_entries(&mut log_store, entries).await.unwrap();

        let all = log_store.try_get_log_entries(1..=10).await.unwrap();
        assert_eq!(all.len(), 10);

        let range = log_store.try_get_log_entries(3..=7).await.unwrap();
        assert_eq!(range.len(), 5);
        assert_eq!(range[0].log_id.index, 3);

        let more: Vec<_> = (11..=15).map(|i| create_entry(1, 1, i)).collect();
        append_entries(&mut log_store, more).await.unwrap();
        assert_eq!(
            log_store.try_get_log_entries(1..=15).await.unwrap().len(),
            15
        );

        log_store.truncate(create_log_id(1, 1, 11)).await.unwrap();
        assert_eq!(
            log_store.try_get_log_entries(1..=15).await.unwrap().len(),
            10
        );
        assert_eq!(
            log_store.try_get_log_entries(11..=15).await.unwrap().len(),
            0
        );

        log_store.purge(create_log_id(1, 1, 5)).await.unwrap();
        assert_eq!(log_store.get_last_purged_().unwrap().unwrap().index, 5);
        let after_purge = log_store.try_get_log_entries(1..=10).await.unwrap();
        assert_eq!(after_purge.len(), 5);
        assert_eq!(after_purge[0].log_id.index, 6);

        let new: Vec<_> = (11..=13).map(|i| create_entry(2, 1, i)).collect();
        append_entries(&mut log_store, new).await.unwrap();
        let final_logs = log_store.try_get_log_entries(6..=13).await.unwrap();
        assert_eq!(final_logs.len(), 8);
        assert_eq!(final_logs[0].log_id.leader_id.term, 1);
        assert_eq!(final_logs[5].log_id.leader_id.term, 2);
    }

    #[test]
    fn test_set_and_get_last_purged() {
        let log_store = create_test_log_store();

        assert!(log_store.get_last_purged_().unwrap().is_none());

        let log_id = create_log_id(1, 1, 100);
        log_store.set_last_purged_(log_id).unwrap();

        let retrieved = log_store.get_last_purged_().unwrap().unwrap();
        assert_eq!(retrieved.leader_id.term, log_id.leader_id.term);
        assert_eq!(retrieved.leader_id.node_id, log_id.leader_id.node_id);
        assert_eq!(retrieved.index, log_id.index);

        let new_log_id = create_log_id(2, 2, 200);
        log_store.set_last_purged_(new_log_id).unwrap();

        let updated = log_store.get_last_purged_().unwrap().unwrap();
        assert_eq!(updated.index, new_log_id.index);
    }

    #[test]
    fn test_set_and_get_committed() {
        let log_store = create_test_log_store();

        assert!(log_store.get_committed_().unwrap().is_none());

        let log_id = create_log_id(1, 1, 100);

        // Debug: test bincode serialization
        let serialized = serialize(&Some(log_id)).unwrap();
        println!("Serialized bytes: {:?}", serialized);
        let deserialized: Option<LogId<TypeConfig>> = deserialize(&serialized).unwrap();
        println!("Original: {:?}", log_id);
        println!("Deserialized: {:?}", deserialized);

        log_store.set_committed_(&Some(log_id)).unwrap();

        // Debug: check what's actually stored in DB
        let raw_bytes = log_store
            .db
            .get_cf(&log_store.store(), key_committed(&log_store.machine))
            .unwrap()
            .unwrap();
        println!("Stored bytes: {:?}", raw_bytes);
        println!("Stored bytes len: {}", raw_bytes.len());

        let retrieved = log_store.get_committed_().unwrap().unwrap();
        println!("Retrieved: {:?}", retrieved);
        assert_eq!(retrieved.leader_id.term, log_id.leader_id.term);
        assert_eq!(retrieved.leader_id.node_id, log_id.leader_id.node_id);
        assert_eq!(retrieved.index, log_id.index);

        let new_log_id = create_log_id(2, 2, 200);
        log_store.set_committed_(&Some(new_log_id)).unwrap();

        let updated = log_store.get_committed_().unwrap().unwrap();
        assert_eq!(updated.index, new_log_id.index);

        log_store.set_committed_(&None).unwrap();
        assert!(log_store.get_committed_().unwrap().is_none());
    }

    #[test]
    fn test_set_and_get_vote() {
        use openraft::Vote;
        let log_store = create_test_log_store();

        assert!(log_store.get_vote_().unwrap().is_none());

        let vote = Vote::new(1, 1);
        log_store.set_vote_(&vote).unwrap();

        let retrieved = log_store.get_vote_().unwrap().unwrap();
        assert_eq!(retrieved.leader_id().term, vote.leader_id().term);
        assert_eq!(retrieved.leader_id().node_id, vote.leader_id().node_id);

        let new_vote = Vote::new(2, 2);
        log_store.set_vote_(&new_vote).unwrap();

        let updated = log_store.get_vote_().unwrap().unwrap();
        assert_eq!(updated.leader_id().term, 2);
        assert_eq!(updated.leader_id().node_id, 2);
    }

    #[test]
    fn test_multiple_machines_isolation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = Arc::new(
            DB::open_cf(&opts, temp_dir.path(), vec![DB_COLUMN_FAMILY_META_RAFT]).unwrap(),
        );

        let log_store1 = LogStore {
            machine: "machine1".to_string(),
            db: db.clone(),
        };
        let log_store2 = LogStore {
            machine: "machine2".to_string(),
            db: db.clone(),
        };

        let log_id1 = create_log_id(1, 1, 100);
        let log_id2 = create_log_id(2, 2, 200);

        log_store1.set_last_purged_(log_id1).unwrap();
        log_store2.set_last_purged_(log_id2).unwrap();

        let result1 = log_store1.get_last_purged_().unwrap().unwrap();
        let result2 = log_store2.get_last_purged_().unwrap().unwrap();

        assert_eq!(result1.index, 100);
        assert_eq!(result2.index, 200);
    }
}
