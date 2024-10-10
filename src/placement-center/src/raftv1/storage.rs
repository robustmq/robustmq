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

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use common_base::error::common::CommonError;
use log::{debug, error};
use raft::eraftpb::HardState;
use raft::prelude::{ConfState, Entry, Snapshot};
use raft::{Error, RaftState, Result as RaftResult, Storage as RaftStorage, StorageError};

use super::rocksdb::RaftMachineStorage;

pub struct RaftRocksDBStorage {
    core: Arc<RwLock<RaftMachineStorage>>,
}

impl RaftRocksDBStorage {
    pub fn new(core: Arc<RwLock<RaftMachineStorage>>) -> Self {
        RaftRocksDBStorage { core }
    }

    pub fn read_lock(&self) -> Result<RwLockReadGuard<'_, RaftMachineStorage>, CommonError> {
        match self.core.read() {
            Ok(object) => Ok(object),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        }
    }

    pub fn write_lock(&self) -> Result<RwLockWriteGuard<'_, RaftMachineStorage>, CommonError> {
        match self.core.write() {
            Ok(object) => Ok(object),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        }
    }
}

impl RaftRocksDBStorage {
    pub fn recovery_snapshot(&self, snapshot: Snapshot) -> Result<(), CommonError> {
        let mut store = self.write_lock()?;
        store.recovery_snapshot(snapshot)
    }

    pub fn append_entrys(&self, entrys: &Vec<Entry>) -> Result<(), CommonError> {
        let mut store = self.write_lock()?;
        store.append_entrys(entrys)
    }

    pub fn commmit_index(&self, idx: u64) -> Result<(), CommonError> {
        let mut store = self.write_lock()?;
        let _ = store.commmit_index(idx);
        Ok(())
    }

    pub fn set_hard_state(&self, hs: HardState) -> Result<(), CommonError> {
        let store = self.write_lock()?;
        let _ = store.save_hard_state(hs.clone());
        Ok(())
    }

    pub fn set_hard_state_comit(&self, hs: u64) -> Result<(), CommonError> {
        let store = self.write_lock()?;
        let _ = store.update_hard_state_commit(hs);
        Ok(())
    }

    pub fn set_conf_state(&self, cs: ConfState) -> Result<(), CommonError> {
        let store = self.write_lock()?;
        let _ = store.save_conf_state(cs);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn create_snapshot(&self) -> Result<(), CommonError> {
        let mut store = self.write_lock()?;
        let _ = store.create_snapshot();
        Ok(())
    }
}

impl RaftStorage for RaftRocksDBStorage {
    /// `initial_state` is called when Raft is initialized. This interface will return a `RaftState`
    /// which contains `HardState` and `ConfState`.
    ///
    /// `RaftState` could be initialized or not. If it's initialized it means the `Storage` is
    /// created with a configuration, and its last index and term should be greater than 0.
    fn initial_state(&self) -> RaftResult<RaftState> {
        let core = match self.read_lock() {
            Ok(obj) => obj,
            Err(e) => return Err(Error::ConfigInvalid(e.to_string())),
        };
        Ok(core.raft_state())
    }

    /// Returns a slice of log entries in the range `[low, high)`.
    /// max_size limits the total size of the log entries returned if not `None`, however
    /// the slice of entries returned will always have length at least 1 if entries are
    /// found in the range.
    ///
    /// Entries are supported to be fetched asynchronously depending on the context. Async is optional.
    /// Storage should check context.can_async() first and decide whether to fetch entries asynchronously
    /// based on its own implementation. If the entries are fetched asynchronously, storage should return
    /// LogTemporarilyUnavailable, and application needs to call `on_entries_fetched(context)` to trigger
    /// re-fetch of the entries after the storage finishes fetching the entries.
    ///
    /// # Panics
    ///
    /// Panics if `high` is higher than `Storage::last_index(&self) + 1`.
    fn entries(
        &self,
        low: u64,
        high: u64,
        _: impl Into<Option<u64>>,
        _: raft::GetEntriesContext,
    ) -> RaftResult<Vec<Entry>> {
        let core = match self.read_lock() {
            Ok(obj) => obj,
            Err(e) => return Err(Error::ConfigInvalid(e.to_string())),
        };

        if low < core.first_index() {
            return Err(Error::Store(StorageError::Compacted));
        }

        if high > core.last_index() + 1 {
            return Err(Error::ConfigInvalid(format!(
                "index out of bound (last: {}, high: {})",
                core.last_index() + 1,
                high
            )));
        }

        let mut entry_list: Vec<Entry> = Vec::new();
        for idx in low..=high {
            let sret = core.entry_by_idx(idx);
            if sret.is_none() {
                continue;
            }
            entry_list.push(sret.unwrap());
        }

        Ok(entry_list)
    }

    /// Returns the term of entry idx, which must be in the range
    /// [first_index()-1, last_index()]. The term of the entry before
    /// first_index is retained for matching purpose even though the
    /// rest of that entry may not be available.
    fn term(&self, idx: u64) -> RaftResult<u64> {
        let core = match self.read_lock() {
            Ok(obj) => obj,
            Err(e) => return Err(Error::ConfigInvalid(e.to_string())),
        };

        if idx < core.first_index() {
            return Err(Error::Store(StorageError::Compacted));
        }

        if idx > core.last_index() {
            return Err(Error::Store(StorageError::Unavailable));
        }

        if let Some(value) = core.entry_by_idx(idx) {
            return Ok(value.term);
        }

        Ok(0)
    }

    /// Returns the index of the first log entry that is possible available via entries, which will
    /// always equal to `truncated index` plus 1.
    ///
    /// New created (but not initialized) `Storage` can be considered as truncated at 0 so that 1
    /// will be returned in this case.
    fn first_index(&self) -> RaftResult<u64> {
        let core = match self.read_lock() {
            Ok(obj) => obj,
            Err(e) => return Err(Error::ConfigInvalid(e.to_string())),
        };
        let fi = core.first_index();
        Ok(fi)
    }

    /// The index of the last entry replicated in the `Storage`.
    fn last_index(&self) -> RaftResult<u64> {
        let core = match self.read_lock() {
            Ok(obj) => obj,
            Err(e) => return Err(Error::ConfigInvalid(e.to_string())),
        };

        let li = core.last_index();
        Ok(li)
    }

    /// Returns the most recent snapshot.
    ///
    /// If snapshot is temporarily unavailable, it should return SnapshotTemporarilyUnavailable,
    /// so raft state machine could know that Storage needs some time to prepare
    /// snapshot and call snapshot later.
    /// A snapshot's index must not less than the `request_index`.
    /// `to` indicates which peer is requesting the snapshot.
    fn snapshot(&self, request_index: u64, to: u64) -> RaftResult<Snapshot> {
        debug!("Node {} requests snapshot data", to);
        let mut core = match self.write_lock() {
            Ok(obj) => obj,
            Err(e) => return Err(Error::ConfigInvalid(e.to_string())),
        };

        if core.raft_snapshot.trigger_snap_unavailable {
            Err(Error::Store(StorageError::SnapshotTemporarilyUnavailable))
        } else {
            let mut snap = match core.get_snapshot() {
                Ok(data) => data,
                Err(e) => {
                    error!("{}", e.to_string());
                    return Err(Error::Store(StorageError::SnapshotTemporarilyUnavailable));
                }
            };
            if snap.get_metadata().index < request_index {
                snap.mut_metadata().index = request_index;
            }
            Ok(snap)
        }
    }
}
