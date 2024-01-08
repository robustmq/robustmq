use crate::storage::rocksdb::RocksDBStorage;
use bincode::{deserialize, serialize};
use common::config::meta::MetaConfig;
use prost::Message as _;
use raft::eraftpb::HardState;
use raft::prelude::ConfState;
use raft::prelude::Entry;
use raft::prelude::Snapshot;
use raft::prelude::SnapshotMetadata;
use raft::Error;
use raft::RaftState;
use raft::Result as RaftResult;
use raft::StorageError;
use std::cmp;
use std::collections::HashMap;

pub struct RaftRocksDBStorageCore {
    rds: RocksDBStorage,
    pub snapshot_metadata: SnapshotMetadata,
    pub trigger_snap_unavailable: bool,
    pub uncommit_index: HashMap<u64, i8>,
    pub snapshot_data: Snapshot,
}

impl RaftRocksDBStorageCore {
    pub fn new(config: &MetaConfig) -> Self {
        let rds = RocksDBStorage::new(config);
        let uncommit_index = HashMap::new();
        let mut rc = RaftRocksDBStorageCore {
            rds,
            snapshot_metadata: SnapshotMetadata::default(),
            trigger_snap_unavailable: false,
            uncommit_index,
            snapshot_data: Snapshot::default(),
        };
        rc.uncommit_index = rc.uncommit_index();
        // rc.init_storage();
        return rc;
    }

    pub fn is_need_init_snapshot(&self) -> bool {
        let cf = self.conf_state();
        let hs = self.hard_state();
        if cf == ConfState::default() && hs == HardState::default() {
            return true;
        }
        return false;
    }

    /// Save HardState information to RocksDB
    pub fn save_conf_state(&self, cs: ConfState) -> Result<(), String> {
        let key = self.key_name_by_conf_state();
        let value = ConfState::encode_to_vec(&cs);
        self.rds.write(self.rds.cf_meta(), &key, &value)
    }

    // Return RaftState
    pub fn raft_state(&self) -> RaftState {
        let shs = self.hard_state();
        let scs = self.conf_state();
        RaftState {
            hard_state: shs,
            conf_state: scs,
        }
    }

    // Save HardState information to RocksDB
    pub fn hard_state(&self) -> HardState {
        let key = self.key_name_by_hard_state();
        let value = self.rds.read::<Vec<u8>>(self.rds.cf_meta(), &key).unwrap();
        if value == None {
            HardState::default()
        } else {
            return HardState::decode(value.unwrap().as_ref())
                .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                .unwrap();
        }
    }

    /// Save HardState information to RocksDB
    pub fn conf_state(&self) -> ConfState {
        let key = self.key_name_by_conf_state();
        let value = self.rds.read::<Vec<u8>>(self.rds.cf_meta(), &key).unwrap();
        if value == None {
            ConfState::default()
        } else {
            return ConfState::decode(value.unwrap().as_ref())
                .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                .unwrap();
        }
    }

    // Obtain the Entry based on the index ID
    pub fn snapshot(&self) -> Snapshot {
        return self.snapshot_data.clone();
    }

    // Example Create a data snapshot for the current system
    pub fn create_snapshot(&mut self) {
        let mut sns = Snapshot::default();

        let hard_state = self.hard_state();
        let conf_state = self.conf_state();

        let meta = sns.mut_metadata();
        meta.set_conf_state(conf_state);
        meta.index = hard_state.commit;
        meta.term = match meta.index.cmp(&self.snapshot_metadata.index) {
            std::cmp::Ordering::Equal => self.snapshot_metadata.term,
            std::cmp::Ordering::Greater => hard_state.term,
            std::cmp::Ordering::Less => {
                panic!(
                    "commit {} < snapshot_metadata.index {}",
                    meta.index, self.snapshot_metadata.index
                );
            }
        };

        let all_data = self.rds.read_all();
        sns.data = serialize(&all_data).unwrap();

        self.snapshot_data = sns;
    }

    // todo
    pub fn commmit_index(&mut self, idx: u64) -> RaftResult<()> {
        self.uncommit_index.remove(&idx);
        self.save_uncommit_index();
        return Ok(());
    }

    pub fn append(&mut self, entrys: &Vec<Entry>) -> RaftResult<()> {
        if entrys.len() == 0 {
            return Ok(());
        }

        let entry_first_index = entrys[0].index;

        let first_index = self.first_index();
        if first_index > entry_first_index {
            panic!(
                "overwrite compacted raft logs, compacted: {}, append: {}",
                first_index - 1,
                entry_first_index,
            );
        }

        let last_index = self.last_index();
        if last_index + 1 < entry_first_index {
            panic!(
                "raft logs should be continuous, last index: {}, new appended: {}",
                last_index, entry_first_index,
            );
        }

        for entry in entrys {
            if entry.data.is_empty() {
                continue;
            }
            let data: Vec<u8> = Entry::encode_to_vec(&entry);

            let key = self.key_name_by_entry(entry.index);
            self.rds.write(self.rds.cf_meta(), &key, &data).unwrap();

            self.save_last_index(entry.index).unwrap();
            self.uncommit_index.insert(entry.index, 1);
        }

        self.save_uncommit_index();

        return Ok(());
    }

    /// Overwrites the contents of this Storage object with those of the given snapshot.
    ///
    /// # Panics
    ///
    /// Panics if the snapshot index is less than the storage's first index.

    pub fn apply_snapshot(&mut self, mut snapshot: Snapshot) -> RaftResult<()> {
        let mut meta = snapshot.take_metadata();
        let index = meta.index;

        if self.first_index() > index {
            return Err(Error::Store(StorageError::SnapshotOutOfDate));
        }

        self.snapshot_metadata = meta.clone();

        // update HardState
        let mut hs = self.hard_state();
        hs.set_term(cmp::max(hs.term, meta.term));
        hs.set_commit(index);
        let _ = self.save_hard_state(hs);

        // update ConfState
        let _ = self.save_conf_state(meta.take_conf_state());

        // todo clear entries
        self.truncate_entry();

        // Restore snapshot data to persistent storage
        self.rds.write_all(snapshot.data.as_ref());
        return Ok(());
    }
}

impl RaftRocksDBStorageCore {
    /// Get the index of the first Entry from RocksDB
    pub fn first_index(&self) -> u64 {
        let key = self.key_name_by_first_index();
        let value = self.rds.read::<u64>(self.rds.cf_meta(), &key).unwrap();
        if value == None {
            self.snapshot_metadata.index + 1
        } else {
            value.unwrap()
        }
    }

    /// Gets the index of the last Entry from RocksDB
    pub fn last_index(&self) -> u64 {
        let key = self.key_name_by_last_index();
        let value = self.rds.read::<u64>(self.rds.cf_meta(), &key).unwrap();
        if value == None {
            self.snapshot_metadata.index
        } else {
            value.unwrap()
        }
    }

    /// Obtain the Entry based on the index ID
    pub fn entry_by_idx(&self, idx: u64) -> Option<Entry> {
        let key = self.key_name_by_entry(idx);
        let value = self.rds.read::<Vec<u8>>(self.rds.cf_meta(), &key).unwrap();
        if value == None {
            return None;
        } else {
            let vl = value.unwrap();
            let et = Entry::decode(vl.as_ref())
                .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                .unwrap();
            return Some(et);
        }
    }

    pub fn save_last_index(&self, index: u64) -> Result<(), String> {
        let key = self.key_name_by_last_index();
        self.rds.write(self.rds.cf_meta(), &key, &index)
    }

    pub fn save_entry(&self, index: u64, value: Entry) -> Result<(), String> {
        let key = self.key_name_by_entry(index);
        self.rds.write(self.rds.cf_meta(), &key, &index)
    }

    pub fn save_first_index(&self, index: u64) -> Result<(), String> {
        let key = self.key_name_by_first_index();
        self.rds.write(self.rds.cf_meta(), &key, &index)
    }

    pub fn truncate_entry(&self) {
        // delete Entry
        let current_first_index = self.first_index();
        let current_last_index = self.last_index();
        for idx in current_first_index..=current_last_index {
            let key = self.key_name_by_entry(idx);
            let _ = self.rds.delete(self.rds.cf_meta(), &key);
        }

        // delete FirstIndex record
        let key = self.key_name_by_first_index();
        let _ = self.rds.delete(self.rds.cf_meta(), &key);

        // delete LastIndex record
        let key = self.key_name_by_last_index();
        let _ = self.rds.delete(self.rds.cf_meta(), &key);
    }

    /// Save HardState information to RocksDB
    pub fn save_hard_state(&self, hs: HardState) -> Result<(), String> {
        let key = self.key_name_by_hard_state();
        let val = HardState::encode_to_vec(&hs);
        self.rds.write(self.rds.cf_meta(), &key, &val)
    }

    ///
    pub fn set_hard_state_commit(&self, commit: u64) -> Result<(), String> {
        let mut hs = self.hard_state();
        hs.commit = commit;
        self.save_hard_state(hs)
    }

    pub fn save_uncommit_index(&self) {
        let ui = self.uncommit_index.clone();
        let val = serialize(&ui).unwrap();
        let key = self.key_name_uncommit();
        let _ = self.rds.write(self.rds.cf_meta(), &key, &val);
    }

    pub fn uncommit_index(&self) -> HashMap<u64, i8> {
        let key = self.key_name_uncommit();
        let value = self.rds.read::<Vec<u8>>(self.rds.cf_meta(), &key).unwrap();
        if value != None {
            return deserialize(value.unwrap().as_ref()).unwrap();
        }
        return HashMap::new();
    }
}

impl RaftRocksDBStorageCore {
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

    fn key_name_by_entry(&self, idx: u64) -> String {
        return format!("metasrv_entry_{}", idx);
    }

    fn key_name_uncommit(&self) -> String {
        return "metasrv_uncommit_index".to_string();
    }
}
