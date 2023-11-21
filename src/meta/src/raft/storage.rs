use common_config::meta::MetaConfig;
use crate::errors::MetaError;
use crate::storage::rocksdb::RocksDBStorage;
use raft::Error;
use raft::StorageError;
use raft::prelude::ConfState;
use raft::prelude::Entry;
use raft::prelude::EntryType;
use raft::prelude::HardState;
use raft::prelude::SnapshotMetadata;
use raft::prelude::Snapshot;
use raft::RaftState;
use raft::Result as RaftResult;
use raft::Storage as RaftStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

#[derive(Clone)]
pub struct RaftRocksDBStorage {
    core: Arc<RwLock<RaftRocksDBStorageCore>>,
}

impl RaftRocksDBStorage {
    pub fn new(config: &MetaConfig) -> Self {
        let core = RaftRocksDBStorageCore::new(config);
        return RaftRocksDBStorage {
            core: Arc::new(RwLock::new(core)),
        };
    }

    pub fn new_with_conf_state<T>(config: &MetaConfig, conf_state: T) -> RaftRocksDBStorage
    where
        ConfState: From<T>,
    {
        let store = RaftRocksDBStorage::new(config);
        store.initialize_with_conf_state(conf_state);
        return store;
    }

    pub fn initialize_with_conf_state<T>(&self, conf_state: T)
    where
        ConfState: From<T>,
    {
        assert!(!self.initial_state().unwrap().initialized());
        self.write_lock().save_conf_state(ConfState::from(conf_state));
    }

    pub fn read_lock(&self) -> RwLockReadGuard<'_, RaftRocksDBStorageCore> {
        self.core.read().unwrap()
    }

    pub fn write_lock(&self) -> RwLockWriteGuard<'_, RaftRocksDBStorageCore> {
        self.core.write().unwrap()
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
        let shs = core.hard_state();
        let scs: SaveRDSConfState = core.conf_state();
        Ok(RaftState {
            hard_state: core.convert_hard_state_from_rds_hs(shs),
            conf_state: core.convert_conf_state_from_rds_cs(scs),
        })
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
        max_size: impl Into<Option<u64>>,
        context: raft::GetEntriesContext,
    ) -> RaftResult<Vec<Entry>> {
        let max_size = max_size.into();
        let mut core = self.read_lock();
        if low < core.first_index(){
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
            let sret = core.get_entry_by_idx(idx);
            entry_list.push(core.convert_entry_from_rds_save_entry(sret).unwrap());
        }
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

        // if idx < core.first_index(){
        //     return Err(MetaError::MetaIndexOutRange);
        // }

        // if idx > core.last_index(){
        //     return Err(Error::new(kind, error));
        // }

        let value = core.get_entry_by_idx(idx);
        return Ok(value.term);
    }

    /// Returns the index of the first log entry that is possible available via entries, which will
    /// always equal to `truncated index` plus 1.
    ///
    /// New created (but not initialized) `Storage` can be considered as truncated at 0 so that 1
    /// will be returned in this case.
    fn first_index(&self) -> RaftResult<u64> {
        Ok(self.read_lock().first_index())
    }

     /// The index of the last entry replicated in the `Storage`.
    fn last_index(&self) -> RaftResult<u64> {
        Ok(self.read_lock().last_index())
    }

    /// Returns the most recent snapshot.
    ///
    /// If snapshot is temporarily unavailable, it should return SnapshotTemporarilyUnavailable,
    /// so raft state machine could know that Storage needs some time to prepare
    /// snapshot and call snapshot later.
    /// A snapshot's index must not less than the `request_index`.
    /// `to` indicates which peer is requesting the snapshot.
    fn snapshot(&self, request_index: u64, to: u64) -> RaftResult<Snapshot> {
        let mut core = self.write_lock();
        if core.trigger_snap_unavailable {
            core.trigger_snap_unavailable = false;
            return Err(Error::Store(StorageError::SnapshotTemporarilyUnavailable))
        } else {
            let mut snap = core.snapshot();
            if snap.get_metadata().index < request_index{
                snap.mut_metadata().index = request_index;
            }
            Ok(snap)
        }
    }
}

pub struct RaftRocksDBStorageCore {
    rds: RocksDBStorage,
    snapshot_metadata: SnapshotMetadata,
    trigger_snap_unavailable: bool,
}

impl RaftRocksDBStorageCore{
    fn new(config: &MetaConfig) -> Self {
        let rds = RocksDBStorage::new(config);
        return RaftRocksDBStorageCore {
            rds: rds,
            snapshot_metadata: SnapshotMetadata::default(),
            trigger_snap_unavailable: false,
        };
    }

    /// Save HardState information to RocksDB
    fn save_hard_state(&self, hs: HardState) -> Result<(), String> {
        let key = self.key_name_by_hard_state();
        let sds_hard_state = SaveRDSHardState {
            term: hs.term,
            vote: hs.vote,
            commit: hs.commit,
        };
        self.rds.write(self.rds.cf_meta(), &key, &sds_hard_state)
    }

    /// Save HardState information to RocksDB
    fn save_conf_state(&self, cs: ConfState) -> Result<(), String> {
        let key = self.key_name_by_conf_state();
        let sds_conf_state = SaveRDSConfState {
            voters: cs.voters,
            learners: cs.learners,
            voters_outgoing: cs.voters_outgoing,
            learners_next: cs.learners_next,
            auto_leave: cs.auto_leave,
        };
        self.rds.write(self.rds.cf_meta(), &key, &sds_conf_state)
    }

    // Save HardState information to RocksDB
    fn hard_state(&self) -> SaveRDSHardState {
        let key = self.key_name_by_hard_state();
        let value = self.rds.read::<SaveRDSHardState>(self.rds.cf_meta(), &key);
        return value.unwrap().unwrap();
    }

    /// Save HardState information to RocksDB
    fn conf_state(&self) -> SaveRDSConfState {
        let key = self.key_name_by_conf_state();
        let value = self.rds.read::<SaveRDSConfState>(self.rds.cf_meta(), &key);
        return value.unwrap().unwrap();
    }

    /// Converts data of type SaveRDSEntry to raft::prelude::Entry
    pub fn convert_entry_from_rds_save_entry(
        &self,
        rds_entry: SaveRDSEntry,
    ) -> Result<Entry, MetaError> {
        let mut ent = Entry::default();

        //What if the Entry Enum options change, there may be a Bug
        let et = match rds_entry.entry_type {
            0_u64 => EntryType::EntryNormal,
            1_u64 => EntryType::EntryConfChange,
            2_u64 => EntryType::EntryConfChangeV2,
            _ => EntryType::EntryNormal,
        };

        ent.entry_type = et;
        ent.term = rds_entry.term;
        ent.index = rds_entry.index;
        ent.data = rds_entry.data;
        ent.context = rds_entry.context;
        ent.sync_log = rds_entry.sync_log;
        return Ok(ent);
    }

    /// Converts data of type SaveRDSConfState to raft::prelude::ConfState
    pub fn convert_conf_state_from_rds_cs(&self, scs: SaveRDSConfState) -> ConfState{
        let mut cs = ConfState::default();
        cs.voters = scs.voters;
        cs.learners = scs.learners;
        cs.voters_outgoing = scs.voters_outgoing;
        cs.learners_next = scs.learners_next;
        cs.auto_leave = scs.auto_leave;
        return cs;
    }

    /// Converts data of type SaveRDSHardState to raft::prelude::HardState
    pub fn convert_hard_state_from_rds_hs(&self, shs:SaveRDSHardState) -> HardState{
        let mut hs = HardState::default();
        hs.term = shs.term;
        hs.vote = shs.vote;
        hs.commit = shs.commit;
        return hs;
    }

    /// Get the index of the first Entry from RocksDB
    fn first_index(&self) -> u64 {
        let key = self.key_name_by_first_index();
        let value = self.rds.read::<u64>(self.rds.cf_meta(), &key);
        return value.unwrap().unwrap();
    }

    /// Gets the index of the last Entry from RocksDB
    fn last_index(&self) -> u64 {
        let key = self.key_name_by_last_index();
        let value = self.rds.read::<u64>(self.rds.cf_meta(), &key);
        return value.unwrap().unwrap();
    }

    /// Obtain the Entry based on the index ID
    fn get_entry_by_idx(&self, idx: u64) -> SaveRDSEntry {
        let key = self.key_name_by_entry(idx);
        let value = self.rds.read::<SaveRDSEntry>(self.rds.cf_meta(), &key);
        value.unwrap().unwrap()
    }

    // Obtain the Entry based on the index ID
    fn snapshot(&self) -> Snapshot{
        let mut sns = Snapshot::default();
        let hard_state = self.hard_state();
        let meta = sns.mut_metadata();
        meta.index = hard_state.commit;
        meta.term = match meta.index.cmp(&self.snapshot_metadata.index){
            std::cmp::Ordering::Equal => self.snapshot_metadata.term,
            std::cmp::Ordering::Greater => self.get_entry_by_idx(meta.index).term,
            std::cmp::Ordering::Less =>{
                panic!(
                    "commit {} < snapshot_metadata.index {}",
                    meta.index, self.snapshot_metadata.index
                );
            },
        };
        meta.set_conf_state(self.convert_conf_state_from_rds_cs(self.conf_state()));
        return sns;
    }

    // pub fn apply_snapshot(&self, mut snapshot:Snapshot) -> Result<()>{
    //     let mut meta = snapshot.take_metadata();
    //     let index = meta.index;

    //     if self.first_index() > index{
    //         return Err(Error::Store(StorageError::SnapshotOutOfDate))
    //     }

    //     return Ok(())
    // }

    fn key_name_by_entry(&self, idx: u64) -> String {
        return format!("metasrv_entry_{}", idx);
    }

    fn key_name_by_first_index(&self) -> String {
        return "metasrv_first_index".to_string();
    }

    fn key_name_by_last_index(&self) -> String {
        return "metasrv_last_index".to_string();
    }

    fn key_name_by_hard_state(&self) -> String {
        return "metasrv_hard_state".to_string();
    }

    fn key_name_by_conf_state(&self) -> String {
        return "metasrv_conf_state".to_string();
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SaveRDSHardState {
    term: u64,
    vote: u64,
    commit: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SaveRDSConfState {
    voters: Vec<u64>,
    learners: Vec<u64>,
    voters_outgoing: Vec<u64>,
    learners_next: Vec<u64>,
    auto_leave: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SaveRDSEntry {
    entry_type: u64,
    term: u64,
    index: u64,
    data: ::bytes::Bytes,
    context: ::bytes::Bytes,
    sync_log: bool,
}

#[cfg(test)]
mod tests {

    // use protobuf::Message;
    // use raft::prelude::ConfState;
    #[test]
    fn encode_pb() {
        // let mut tt = ConfState::default();
        // Message::write_to_bytes(&tt);
        // let mut buf = vec![];
        // tt.encode(&mut buf).unwrap();
    }
}
