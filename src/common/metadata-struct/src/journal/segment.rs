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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JournalSegment {
    pub shard_name: String,
    pub segment_seq: u32,
    pub replica: Vec<JournalSegmentNode>,
    pub status: JournalSegmentStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JournalSegmentNode {
    pub node_id: u64,
    pub data_fold: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum JournalSegmentStatus {
    CREATE,
    AVTIVE,
    BLOCKED,
}
