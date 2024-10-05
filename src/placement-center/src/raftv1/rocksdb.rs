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

use crate::storage::keys::key_name_by_conf_state;
use crate::storage::keys::key_name_by_entry;
use crate::storage::keys::key_name_by_first_index;
use crate::storage::keys::key_name_by_hard_state;
use crate::storage::keys::key_name_by_last_index;
use crate::storage::keys::key_name_uncommit;
use crate::storage::keys::key_name_uncommit_prefix;
use crate::storage::rocksdb::RocksDBEngine;
use bincode::serialize;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use log::debug;
use log::error;
use log::info;
use prost::Message as _;
use raft::eraftpb::HardState;
use raft::prelude::ConfState;
use raft::prelude::Entry;
use raft::prelude::Snapshot;
use raft::RaftState;
use raft::Result as RaftResult;
use rocksdb::DEFAULT_COLUMN_FAMILY_NAME;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;

use super::snapshot::RaftSnapshot;

#[derive(Serialize, Deserialize)]
pub struct RaftUncommitData {
    pub index: u64,
    pub create_time: u64,
}

pub struct RaftMachineStorage {
    pub raft_snapshot: RaftSnapshot,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl RaftMachineStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let rc = RaftMachineStorage {
            rocksdb_engine_handler,
            raft_snapshot: RaftSnapshot::new(),
        };
        return rc;
    }
}

impl RaftMachineStorage {
    pub fn append_entrys(&mut self, entrys: &Vec<Entry>) -> Result<(), CommonError> {
        if entrys.len() == 0 {
            return Ok(());
        }

        let entry_first_index = entrys[0].index;

        let first_index = self.first_index();
        if first_index > entry_first_index {
            return Err(CommonError::CommmonError(format!(
                "overwrite compacted raft logs, compacted: {}, append: {}",
                first_index - 1,
                entry_first_index,
            )));
        }

        let last_index = self.last_index();
        if last_index + 1 < entry_first_index {
            return Err(CommonError::CommmonError(format!(
                "raft logs should be continuous, last index: {}, new appended: {}",
                last_index, entry_first_index,
            )));
        }

        for entry in entrys {
            debug!(">> save entry index:{}, value:{:?}", entry.index, entry);
            let data: Vec<u8> = Entry::encode_to_vec(&entry);
            let key = key_name_by_entry(entry.index);
            self.rocksdb_engine_handler
                .write(
                    self.rocksdb_engine_handler
                        .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                        .unwrap(),
                    &key,
                    &data,
                )
                .unwrap();
            self.save_uncommit_index(entry.index)?;
            self.save_last_index(entry.index)?;
        }

        return Ok(());
    }
}

impl RaftMachineStorage {
    pub fn raft_state(&self) -> RaftState {
        let shs = self.hard_state();
        let scs = self.conf_state();
        RaftState {
            hard_state: shs,
            conf_state: scs,
        }
    }

    pub fn hard_state(&self) -> HardState {
        let key = key_name_by_hard_state();
        let value = self
            .rocksdb_engine_handler
            .read::<Vec<u8>>(
                self.rocksdb_engine_handler
                    .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                    .unwrap(),
                &key,
            )
            .unwrap();
        if value == None {
            HardState::default()
        } else {
            return HardState::decode(value.unwrap().as_ref()).unwrap();
        }
    }

    pub fn conf_state(&self) -> ConfState {
        let key = key_name_by_conf_state();
        let value = self
            .rocksdb_engine_handler
            .read::<Vec<u8>>(
                self.rocksdb_engine_handler
                    .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                    .unwrap(),
                &key,
            )
            .unwrap();
        if value.is_none() {
            ConfState::default()
        } else {
            return ConfState::decode(value.unwrap().as_ref())
                .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                .unwrap();
        }
    }

    pub fn first_index(&self) -> u64 {
        let key = key_name_by_first_index();
        match self.rocksdb_engine_handler.read::<u64>(
            self.rocksdb_engine_handler
                .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                .unwrap(),
            &key,
        ) {
            Ok(value) => {
                if let Some(fi) = value {
                    fi
                } else {
                    self.raft_snapshot.snapshot_metadata.index + 1
                }
            }
            Err(e) => {
                error!("Failed to read the first index. The failure message is {}, and the current snapshot index is {}",e, self.raft_snapshot.snapshot_metadata.index);
                self.raft_snapshot.snapshot_metadata.index + 1
            }
        }
    }

    pub fn last_index(&self) -> u64 {
        let key = key_name_by_last_index();
        match self.rocksdb_engine_handler.read::<u64>(
            self.rocksdb_engine_handler
                .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                .unwrap(),
            &key,
        ) {
            Ok(value) => {
                if let Some(li) = value {
                    li
                } else {
                    self.raft_snapshot.snapshot_metadata.index
                }
            }
            Err(e) => {
                error!("Failed to read the last index. The failure message is {}, and the current snapshot index is {}",e, self.raft_snapshot.snapshot_metadata.index);
                self.raft_snapshot.snapshot_metadata.index
            }
        }
    }

    pub fn entry_by_idx(&self, idx: u64) -> Option<Entry> {
        let key = key_name_by_entry(idx);
        match self.rocksdb_engine_handler.read::<Vec<u8>>(
            self.rocksdb_engine_handler
                .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                .unwrap(),
            &key,
        ) {
            Ok(value) => {
                if let Some(vl) = value {
                    let et = Entry::decode(vl.as_ref())
                        .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                        .unwrap();
                    return Some(et);
                }
            }
            Err(e) => error!(
                "Failed to read entry. The failure information is {}, and the current index is {}",
                e, idx
            ),
        }
        return None;
    }

    pub fn save_last_index(&self, index: u64) -> Result<(), CommonError> {
        let key = key_name_by_last_index();
        self.rocksdb_engine_handler
            .write(
                self.rocksdb_engine_handler
                    .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                    .unwrap(),
                &key,
                &index,
            )
            .unwrap();
        return Ok(());
    }

    pub fn save_first_index(&self, index: u64) -> Result<(), String> {
        let key = key_name_by_first_index();
        self.rocksdb_engine_handler.write(
            self.rocksdb_engine_handler
                .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                .unwrap(),
            &key,
            &index,
        )
    }

    pub fn save_conf_state(&self, cs: ConfState) -> Result<(), String> {
        let key = key_name_by_conf_state();
        let value = ConfState::encode_to_vec(&cs);
        self.rocksdb_engine_handler.write(
            self.rocksdb_engine_handler
                .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                .unwrap(),
            &key,
            &value,
        )
    }

    pub fn save_hard_state(&self, hs: HardState) -> Result<(), String> {
        let key = key_name_by_hard_state();
        let val = HardState::encode_to_vec(&hs);
        self.rocksdb_engine_handler.write(
            self.rocksdb_engine_handler
                .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                .unwrap(),
            &key,
            &val,
        )
    }

    pub fn update_hard_state_commit(&self, commit: u64) -> Result<(), String> {
        let mut hs = self.hard_state();
        hs.commit = commit;
        self.save_hard_state(hs)
    }
}

impl RaftMachineStorage {
    pub fn recovery_snapshot(&mut self, snapshot: Snapshot) -> Result<(), CommonError> {
        info!(
            "recovery snapshot,term:{},index:{}",
            snapshot.get_metadata().get_term(),
            snapshot.get_metadata().get_index()
        );
        return self.raft_snapshot.recovery_snapshot();
    }

    pub fn get_snapshot(&mut self) -> Result<Snapshot, CommonError> {
        // self.create_snapshot();
        // let key = key_name_snapshot();
        // let value = self
        //     .rocksdb_engine_handler
        //     .read::<Vec<u8>>(self.rocksdb_engine_handler
        // .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
        // .unwrap(), &key)
        //     .unwrap();
        // if value.is_none() {
        //     Snapshot::default()
        // } else {
        //     return Snapshot::decode(value.unwrap().as_ref())
        //         .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
        //         .unwrap();
        // }
        return Ok(Snapshot::default());
    }

    // Example Create a data snapshot for the current system
    pub fn create_snapshot(&mut self) -> Result<(), CommonError> {
        return self.raft_snapshot.create_snapshot();
    }
}

impl RaftMachineStorage {
    pub fn commmit_index(&mut self, idx: u64) -> RaftResult<()> {
        let entry = self.entry_by_idx(idx);
        if entry.is_none() {
            info!("commit_to {} but the entry does not exist", idx);
        }

        debug!(">> commit entry index:{}", idx);

        self.remove_uncommit_index(idx).unwrap();

        let mut hs = self.hard_state();
        hs.commit = idx;
        hs.term = entry.unwrap().get_term();
        let _ = self.save_hard_state(hs);
        return Ok(());
    }

    pub fn remove_uncommit_index(&self, idx: u64) -> Result<(), CommonError> {
        let key = key_name_uncommit(idx);
        let _ = self.rocksdb_engine_handler.delete(
            self.rocksdb_engine_handler
                .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                .unwrap(),
            &key,
        );
        return Ok(());
    }

    pub fn save_uncommit_index(&self, idx: u64) -> Result<(), CommonError> {
        let data = RaftUncommitData {
            index: idx,
            create_time: now_second(),
        };
        match serialize(&data) {
            Ok(da) => {
                let key = key_name_uncommit(idx);
                let _ = self.rocksdb_engine_handler.write(
                    self.rocksdb_engine_handler
                        .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
                        .unwrap(),
                    &key,
                    &da,
                );
                return Ok(());
            }
            Err(e) => {
                return Err(CommonError::CommmonError(e.to_string()));
            }
        };
    }

    pub fn all_uncommit_index(&self) -> Vec<RaftUncommitData> {
        let key = key_name_uncommit_prefix();
        let cf = self
            .rocksdb_engine_handler
            .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
            .unwrap();
        let results = self.rocksdb_engine_handler.read_prefix(&cf, &key);
        let mut data_list = Vec::new();
        for raw in results {
            for (_, v) in raw {
                match serde_json::from_slice::<RaftUncommitData>(v.as_ref()) {
                    Ok(v) => data_list.push(v),
                    Err(_) => {
                        continue;
                    }
                }
            }
        }

        return data_list;
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::remove_dir_all, sync::Arc};

    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    use super::RaftMachineStorage;
    use common_base::config::placement_center::placement_center_test_conf;

    #[test]
    fn write_read_test() {
        let conf = placement_center_test_conf();

        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(
            &conf.rocksdb.data_path,
            conf.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let rds = RaftMachineStorage::new(rocksdb_engine_handler);

        let first_index = 1;
        let _ = rds.save_first_index(first_index);
        let read_first_index = rds.first_index();
        assert_eq!(first_index, read_first_index);

        let last_index = 2;
        let _ = rds.save_last_index(last_index);
        let last_index = rds.last_index();
        assert_eq!(last_index, last_index);

        remove_dir_all(conf.rocksdb.data_path).unwrap();
    }
}
