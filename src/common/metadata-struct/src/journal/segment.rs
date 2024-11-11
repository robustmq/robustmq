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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct JournalSegment {
    pub cluster_name: String,
    pub namespace: String,
    pub shard_name: String,
    pub segment_seq: u32,
    pub replicas: Vec<Replica>,
    pub leader_epoch: u32,
    pub leader: u64,
    pub isr: Vec<Replica>,
    pub status: SegmentStatus,
    pub config: SegmentConfig,
}

impl JournalSegment {
    pub fn allow_read(&self) -> bool {
        self.status == SegmentStatus::Write
    }

    pub fn get_fold(&self, node_id: u64) -> Option<String> {
        for rep in self.replicas.clone() {
            if rep.node_id == node_id {
                return Some(rep.fold);
            }
        }
        None
    }

    pub fn name(&self) -> String {
        format!(
            "{},{},{},{}",
            self.cluster_name, self.namespace, self.shard_name, self.segment_seq
        )
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Replica {
    pub replica_seq: u64,
    pub node_id: u64,
    pub fold: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SegmentStatus {
    #[default]
    Idle,
    PreWrite,
    Write,
    PreSealUp,
    SealUp,
    PreDelete,
    Deleteing,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SegmentConfig {
    pub max_segment_size: u64,
}
