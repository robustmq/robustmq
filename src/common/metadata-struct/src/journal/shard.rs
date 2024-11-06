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

pub struct JournalShard {
    pub shard_uid: String,
    pub cluster_name: String,
    pub namespace: String,
    pub shard_name: String,
    pub replica: u32,
    pub start_segment_seq: u32,
    pub active_segment_seq: u32,
    pub last_segment_seq: u32,
    pub status: JournalShardStatus,
    pub create_time: u128,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalShardStatus {
    #[default]
    Run,
    PrepareDelete,
    Deleteing,
    Delete,
}
