use crate::storage::rocksdb::RocksDBStorage;
use common::config::meta::MetaConfig;

use raft::prelude::ConfState;
use raft::prelude::Entry;
use raft::prelude::Snapshot;
use raft::prelude::SnapshotMetadata;
use raft::Error;
use raft::RaftState;
use raft::Result as RaftResult;
use raft::StorageError;
use raft_proto::eraftpb::HardState;
use std::cmp;
use super::data::SaveRDSConfState;
use super::data::SaveRDSEntry;
use super::data::SaveRDSHardState;
use super::data::convert_conf_state_from_rds_cs;
use super::data::convert_hard_state_from_rds_hs;

pub struct RaftRocksDBStorageCore {
    raft_state: RaftState,
    rds: RocksDBStorage,
    pub snapshot_metadata: SnapshotMetadata,
    pub trigger_snap_unavailable: bool,
}

impl RaftRocksDBStorageCore {
    pub fn new(config: &MetaConfig) -> Self {
        let rds = RocksDBStorage::new(config);
        return RaftRocksDBStorageCore {
            raft_state: RaftState {
                ..Default::default()
            },
            rds: rds,
            snapshot_metadata: SnapshotMetadata::default(),
            trigger_snap_unavailable: false,
        };
    }

    /// Save HardState information to RocksDB
    pub fn save_hard_state(&self, hs: &HardState) -> Result<(), String> {
        let key = self.key_name_by_hard_state();
        let sds_hard_state = SaveRDSHardState {
            term: hs.term,
            vote: hs.vote,
            commit: hs.commit,
        };
        self.rds.write(self.rds.cf_meta(), &key, &sds_hard_state)
    }

    /// 
    pub fn set_hard_state_commit(&self, commit:u64) -> Result<(),String>{
        let mut hs = self.hard_state();
        hs.commit = commit;

        let new_hs = convert_hard_state_from_rds_hs(hs);
        self.save_hard_state(&new_hs)
    }

    /// Save HardState information to RocksDB
    pub fn save_conf_state(&self, cs: ConfState) -> Result<(), String> {
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
    pub fn hard_state(&self) -> SaveRDSHardState {
        let key = self.key_name_by_hard_state();
        let value = self.rds.read::<SaveRDSHardState>(self.rds.cf_meta(), &key);
        return value.unwrap().unwrap();
    }

    /// Save HardState information to RocksDB
    pub fn conf_state(&self) -> SaveRDSConfState {
        let key = self.key_name_by_conf_state();
        let value = self.rds.read::<SaveRDSConfState>(self.rds.cf_meta(), &key);
        return value.unwrap().unwrap();
    }

    /// Get the index of the first Entry from RocksDB
    pub fn first_index(&self) -> u64 {
        let key = self.key_name_by_first_index();
        let value = self.rds.read::<u64>(self.rds.cf_meta(), &key);
        return value.unwrap().unwrap();
    }

    /// Gets the index of the last Entry from RocksDB
    pub fn last_index(&self) -> u64 {
        let key = self.key_name_by_last_index();
        let value = self.rds.read::<u64>(self.rds.cf_meta(), &key);
        return value.unwrap().unwrap();
    }

    /// Obtain the Entry based on the index ID
    pub fn get_entry_by_idx(&self, idx: u64) -> SaveRDSEntry {
        let key = self.key_name_by_entry(idx);
        let value = self.rds.read::<SaveRDSEntry>(self.rds.cf_meta(), &key);
        value.unwrap().unwrap()
    }

    // Obtain the Entry based on the index ID
    pub fn snapshot(&self) -> Snapshot {
        let mut sns = Snapshot::default();
        let hard_state = self.hard_state();
        let meta = sns.mut_metadata();
        meta.index = hard_state.commit;
        meta.term = match meta.index.cmp(&self.snapshot_metadata.index) {
            std::cmp::Ordering::Equal => self.snapshot_metadata.term,
            std::cmp::Ordering::Greater => self.get_entry_by_idx(meta.index).term,
            std::cmp::Ordering::Less => {
                panic!(
                    "commit {} < snapshot_metadata.index {}",
                    meta.index, self.snapshot_metadata.index
                );
            }
        };
        meta.set_conf_state(convert_conf_state_from_rds_cs(self.conf_state()));
        return sns;
    }

    pub fn append(&mut self, entrys: &Vec<Entry>) -> RaftResult<()> {
        if entrys.len() == 0 {
            return Ok(());
        }

        if self.first_index() > entrys[0].index {
            panic!(
                "overwrite compacted raft logs, compacted: {}, append: {}",
                self.first_index() - 1,
                entrys[0].index,
            );
        }

        if self.last_index() + 1 < entrys[0].index {
            panic!(
                "raft logs should be continuous, last index: {}, new appended: {}",
                self.last_index(),
                entrys[0].index,
            );
        }

        for entry in entrys {
            let key = self.key_name_by_entry(entry.index);
            let sre = SaveRDSEntry{
                entry_type: entry.entry_type as u64,
                term: entry.term,
                index: entry.index,
                data: entry.data.clone(),
                context: entry.context.clone(),
                sync_log: entry.sync_log,
            };
            self.rds.write(self.rds.cf_meta(), &key, &sre).unwrap();
        }

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

        // update hardstate
        let cur_hs = self.hard_state();
        let mut hs = HardState::new();
        hs.set_term(cmp::max(cur_hs.term, meta.term));
        hs.set_commit(index);
        let _ = self.save_hard_state(&hs);

        // todo clear entries

        // update conf state
        let _ = self.save_conf_state(meta.take_conf_state());
        return Ok(());
    }

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
