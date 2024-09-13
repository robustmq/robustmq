// Copyright 2023 RobustMQ Team
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

use crate::storage::placement::raft::RaftMachineStorage;
use log::info;
use raft::eraftpb::HardState;
use raft::prelude::ConfState;
use raft::prelude::Entry;
use raft::prelude::Snapshot;
use raft::Error;
use raft::RaftState;
use raft::Result as RaftResult;
use raft::Storage as RaftStorage;
use raft::StorageError;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

pub struct RaftRocksDBStorage {
    core: Arc<RwLock<RaftMachineStorage>>,
}

impl RaftRocksDBStorage {
    pub fn new(core: Arc<RwLock<RaftMachineStorage>>) -> Self {
        return RaftRocksDBStorage { core };
    }

    #[allow(dead_code)]
    pub fn new_with_conf_state<T>(
        core: Arc<RwLock<RaftMachineStorage>>,
        conf_state: T,
    ) -> RaftRocksDBStorage
    where
        ConfState: From<T>,
    {
        let store = RaftRocksDBStorage::new(core);
        store.initialize_with_conf_state(conf_state);
        return store;
    }

    #[allow(dead_code)]
    pub fn initialize_with_conf_state<T>(&self, conf_state: T)
    where
        ConfState: From<T>,
    {
        assert!(!self.initial_state().unwrap().initialized());
        let _ = self
            .write_lock()
            .save_conf_state(ConfState::from(conf_state));
    }

    pub fn read_lock(&self) -> RwLockReadGuard<'_, RaftMachineStorage> {
        self.core.read().unwrap()
    }

    pub fn write_lock(&self) -> RwLockWriteGuard<'_, RaftMachineStorage> {
        self.core.write().unwrap()
    }
}

impl RaftRocksDBStorage {
    pub fn apply_snapshot(&mut self, snapshot: Snapshot) -> RaftResult<()> {
        let mut store = self.core.write().unwrap();
        let _ = store.apply_snapshot(snapshot);
        Ok(())
    }

    pub fn append(&mut self, entrys: &Vec<Entry>) -> RaftResult<()> {
        let mut store = self.core.write().unwrap();
        let _ = store.append(entrys);
        return Ok(());
    }

    pub fn commmit_index(&mut self, idx: u64) -> RaftResult<()> {
        let mut store = self.core.write().unwrap();
        let _ = store.commmit_index(idx);
        return Ok(());
    }

    pub fn set_hard_state(&mut self, hs: HardState) -> RaftResult<()> {
        let store = self.core.write().unwrap();
        let _ = store.save_hard_state(hs.clone());
        return Ok(());
    }

    pub fn set_hard_state_comit(&mut self, hs: u64) -> RaftResult<()> {
        let store = self.core.write().unwrap();
        let _ = store.set_hard_state_commit(hs);
        return Ok(());
    }

    pub fn set_conf_state(&mut self, cs: ConfState) -> RaftResult<()> {
        let store = self.core.write().unwrap();
        let _ = store.save_conf_state(cs);
        return Ok(());
    }

    pub fn create_snapshot(&mut self) -> RaftResult<()> {
        let mut store = self.core.write().unwrap();
        let _ = store.create_snapshot();
        return Ok(());
    }
}

impl RaftStorage for RaftRocksDBStorage {
    /// `initial_state` is called when Raft is initialized. This interface will return a `RaftState`
    /// which contains `HardState` and `ConfState`.
    ///
    /// `RaftState` could be initialized or not. If it's initialized it means the `Storage` is
    /// created with a configuration, and its last index and term should be greater than 0.
    fn initial_state(&self) -> RaftResult<RaftState> {
        let core = self.read_lock();
        return Ok(core.raft_state());
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
        let core = self.read_lock();
        if low < core.first_index() {
            return Err(Error::Store(StorageError::Compacted));
        }

        if high > core.last_index() + 1 {
            panic!(
                "index out of bound (last: {}, high: {})",
                core.last_index() + 1,
                high
            )
        }

        let mut entry_list: Vec<Entry> = Vec::new();
        for idx in low..=high {
            let sret = core.entry_by_idx(idx);
            if sret == None {
                continue;
            }
            entry_list.push(sret.unwrap());
        }

        // todo limit size

        return Ok(entry_list);
    }

    /// Returns the term of entry idx, which must be in the range
    /// [first_index()-1, last_index()]. The term of the entry before
    /// first_index is retained for matching purpose even though the
    /// rest of that entry may not be available.
    fn term(&self, idx: u64) -> RaftResult<u64> {
        let core = self.read_lock();

        if idx == core.snapshot_metadata.index {
            return Ok(core.snapshot_metadata.index);
        }

        if idx < core.first_index() {
            return Err(Error::Store(StorageError::Compacted));
        }

        if idx > core.last_index() {
            return Err(Error::Store(StorageError::Unavailable));
        }

        if let Some(value) = core.entry_by_idx(idx) {
            return Ok(value.term);
        }

        return Ok(core.snapshot_metadata.term);
    }

    /// Returns the index of the first log entry that is possible available via entries, which will
    /// always equal to `truncated index` plus 1.
    ///
    /// New created (but not initialized) `Storage` can be considered as truncated at 0 so that 1
    /// will be returned in this case.
    fn first_index(&self) -> RaftResult<u64> {
        let core = self.read_lock();
        let fi = core.first_index();
        Ok(fi)
    }

    /// The index of the last entry replicated in the `Storage`.
    fn last_index(&self) -> RaftResult<u64> {
        let core = self.read_lock();
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
        info!("Node {} requests snapshot data", to);
        let mut core = self.write_lock();
        if core.trigger_snap_unavailable {
            return Err(Error::Store(StorageError::SnapshotTemporarilyUnavailable));
        } else {
            let mut snap = core.snapshot();
            if snap.get_metadata().index < request_index {
                snap.mut_metadata().index = request_index;
            }
            Ok(snap)
        }
    }
}
