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
use openraft::storage::{IOFlushed, RaftLogStorage};
use openraft::{
    AnyError, Entry, ErrorSubject, ErrorVerb, LogId, LogState, OptionalSend, RaftLogReader,
    StorageError, Vote,
};
use rocksdb::{BoundColumnFamily, Direction, WriteBatch, DB};
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
    fn store(&self) -> Arc<BoundColumnFamily<'_>> {
        self.db.cf_handle(DB_COLUMN_FAMILY_META_RAFT).unwrap()
    }

    fn flush(
        &self,
        subject: ErrorSubject<TypeConfig>,
        verb: ErrorVerb,
    ) -> Result<(), StorageError<TypeConfig>> {
        self.db
            .flush_wal(true)
            .map_err(|e| StorageError::new(subject, verb, AnyError::new(&e)))?;
        Ok(())
    }

    // Last Purged Log Id
    fn get_last_purged_(&self) -> StorageResult<Option<LogId<TypeConfig>>> {
        Ok(self
            .db
            .get_cf(&self.store(), key_last_purged_log_id(&self.machine))
            .map_err(|e| StorageError::read(&e))?
            .and_then(|v| serde_json::from_slice(&v).ok()))
    }

    fn set_last_purged_(&self, log_id: LogId<TypeConfig>) -> StorageResult<()> {
        self.db
            .put_cf(
                &self.store(),
                key_last_purged_log_id(&self.machine),
                serde_json::to_vec(&log_id).unwrap().as_slice(),
            )
            .map_err(|e| StorageError::write(&e))?;

        self.flush(ErrorSubject::Store, ErrorVerb::Write)?;
        Ok(())
    }

    // Committed
    fn set_committed_(
        &self,
        committed: &Option<LogId<TypeConfig>>,
    ) -> Result<(), StorageError<TypeConfig>> {
        let json = serde_json::to_vec(committed).unwrap();

        self.db
            .put_cf(&self.store(), key_committed(&self.machine), json)
            .map_err(|e| StorageError::write(&e))?;

        self.flush(ErrorSubject::Store, ErrorVerb::Write)?;
        Ok(())
    }

    fn get_committed_(&self) -> StorageResult<Option<LogId<TypeConfig>>> {
        Ok(self
            .db
            .get_cf(&self.store(), key_committed(&self.machine))
            .map_err(|e| StorageError::read(&e))?
            .and_then(|v| serde_json::from_slice(&v).ok()))
    }

    // Vote
    fn set_vote_(&self, vote: &Vote<TypeConfig>) -> StorageResult<()> {
        self.db
            .put_cf(
                &self.store(),
                key_vote(&self.machine),
                serde_json::to_vec(vote).unwrap(),
            )
            .map_err(|e| StorageError::write_vote(&e))?;

        self.flush(ErrorSubject::Vote, ErrorVerb::Write)?;
        Ok(())
    }

    fn get_vote_(&self) -> StorageResult<Option<Vote<TypeConfig>>> {
        Ok(self
            .db
            .get_cf(&self.store(), key_vote(&self.machine))
            .map_err(|e| StorageError::write_vote(&e))?
            .and_then(|v| serde_json::from_slice(&v).ok()))
    }
}

impl RaftLogReader<TypeConfig> for LogStore {
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> StorageResult<Vec<Entry<TypeConfig>>> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(x) => {
                key_raft_log(&self.machine, *x).map_err(|e| StorageError::read_logs(&e))?
            }
            std::ops::Bound::Excluded(x) => {
                key_raft_log(&self.machine, *x + 1).map_err(|e| StorageError::read_logs(&e))?
            }
            std::ops::Bound::Unbounded => {
                key_raft_log(&self.machine, 0).map_err(|e| StorageError::read_logs(&e))?
            }
        };

        self.db
            .iterator_cf(
                &self.store(),
                rocksdb::IteratorMode::From(&start, Direction::Forward),
            )
            .map(|res| {
                let (key, val) = res.unwrap();
                let entry: StorageResult<Entry<_>> =
                    serde_json::from_slice(&val).map_err(|e| StorageError::read_logs(&e));
                let id = raft_log_key_to_id(&self.machine, &key).unwrap();
                (id, entry)
            })
            .take_while(|(id, _)| range.contains(id))
            .map(|x| x.1)
            .collect()
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<TypeConfig>>, StorageError<TypeConfig>> {
        self.get_vote_()
    }
}

impl RaftLogStorage<TypeConfig> for LogStore {
    type LogReader = Self;

    async fn get_log_state(&mut self) -> StorageResult<LogState<TypeConfig>> {
        let last = self
            .db
            .iterator_cf(&self.store(), rocksdb::IteratorMode::End)
            .next()
            .and_then(|res| {
                let (_, ent) = res.unwrap();
                Some(
                    serde_json::from_slice::<Entry<TypeConfig>>(&ent)
                        .ok()?
                        .log_id,
                )
            });

        let last_purged_log_id = self.get_last_purged_()?;

        let last_log_id = match last {
            None => last_purged_log_id,
            Some(x) => Some(x),
        };
        Ok(LogState {
            last_purged_log_id,
            last_log_id,
        })
    }

    async fn save_committed(
        &mut self,
        _committed: Option<LogId<TypeConfig>>,
    ) -> Result<(), StorageError<TypeConfig>> {
        self.set_committed_(&_committed)?;
        Ok(())
    }

    async fn read_committed(
        &mut self,
    ) -> Result<Option<LogId<TypeConfig>>, StorageError<TypeConfig>> {
        let c = self.get_committed_()?;
        Ok(c)
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

        // Collect all entries into WriteBatch for batch write
        for entry in entries {
            let id = key_raft_log(&self.machine, entry.log_id.index)
                .map_err(|e| StorageError::read_logs(&e))?;
            let serialized =
                serde_json::to_vec(&entry).map_err(|e| StorageError::write_logs(&e))?;

            batch.put_cf(&self.store(), id, serialized);
            entry_count += 1;
        }

        // Perform batch write if there are entries
        if entry_count > 0 {
            self.db
                .write(batch)
                .map_err(|e| StorageError::write_logs(&e))?;
        }

        callback.io_completed(Ok(()));
        Ok(())
    }

    async fn truncate(&mut self, log_id: LogId<TypeConfig>) -> StorageResult<()> {
        let from =
            key_raft_log(&self.machine, log_id.index).map_err(|e| StorageError::read_logs(&e))?;
        let to = key_raft_log(&self.machine, 0xff_ff_ff_ff_ff_ff_ff_ff)
            .map_err(|e| StorageError::read_logs(&e))?;
        self.db
            .delete_range_cf(&self.store(), &from, &to)
            .map_err(|e| StorageError::write_logs(&e))
    }

    async fn purge(&mut self, log_id: LogId<TypeConfig>) -> Result<(), StorageError<TypeConfig>> {
        self.set_last_purged_(log_id)?;
        let from = key_raft_log(&self.machine, 0).map_err(|e| StorageError::read_logs(&e))?;
        let to = key_raft_log(&self.machine, log_id.index + 1)
            .map_err(|e| StorageError::read_logs(&e))?;
        self.db
            .delete_range_cf(&self.store(), &from, &to)
            .map_err(|e| StorageError::write_logs(&e))
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }
}
