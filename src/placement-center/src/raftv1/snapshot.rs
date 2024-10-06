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

use common_base::error::common::CommonError;
use raft::prelude::SnapshotMetadata;
pub struct RaftSnapshot {
    pub trigger_snap_unavailable: bool,
    pub snapshot_metadata: SnapshotMetadata,
}

impl RaftSnapshot {
    pub fn new() -> Self {
        return RaftSnapshot {
            snapshot_metadata: SnapshotMetadata::default(),
            trigger_snap_unavailable: false,
        };
    }

    pub fn recovery_snapshot(&self) -> Result<(), CommonError> {
        println!("{}","recovery_snapshot");
        // let mut meta = snapshot.take_metadata();
        // let index = meta.index;

        // if self.first_index() > index {
        //     return Err(Error::Store(StorageError::SnapshotOutOfDate));
        // }

        // self.snapshot_metadata = meta.clone();

        // // Restore snapshot data to persistent storage
        // self.write_all(snapshot.data.as_ref());

        // // update HardState
        // let mut hs = self.hard_state();
        // hs.set_term(cmp::max(hs.term, meta.term));
        // hs.set_commit(index);
        // let _ = self.save_hard_state(hs);

        // // update ConfState
        // let _ = self.save_conf_state(meta.take_conf_state());
        return Ok(());
    }

    #[warn(dead_code)]
    pub fn create_snapshot(&self) -> Result<(), CommonError> {
        println!("{}","create_snapshot");
        // let mut sns = Snapshot::default();

        // // create snapshot metadata
        // let meta = self.create_snapshot_metadata();
        // sns.set_metadata(meta.clone());

        // // create snapshot data

        // let all_data = self.rocksdb_engine_handler.read_all();
        // sns.set_data(serialize(&all_data).unwrap());

        // // update value
        // let _ = self.save_first_index(meta.get_index());

        // //todo clear < first_index entry log

        // self.save_snapshot_data(sns);
        // self.snapshot_metadata = meta.clone();
        return Ok(());
    }

    #[warn(dead_code)]
    pub fn create_snapshot_metadata(&self) -> Result<SnapshotMetadata, CommonError> {
        println!("{}","create_snapshot_metadata");
        // let hard_state = self.hard_state();
        // let conf_state = self.conf_state();

        let meta: SnapshotMetadata = SnapshotMetadata::default();
        // meta.set_conf_state(conf_state);
        // meta.set_index(hard_state.commit);
        // meta.set_term(hard_state.term);
        return Ok(meta);
    }
}
