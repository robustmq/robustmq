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

#[derive(Serialize, Deserialize, Debug)]
pub struct JournalSegment {
    shard_name: String,
    segment_seq: u32,
    start_offset: u64,
    end_offset: u64,
    replica: Vec<JournalSegmentNode>,
    status: JournalSegmentStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JournalSegmentNode {
    node_id: u32,
    data_fold: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JournalSegmentStatus {
    CREATE,
    AVTIVE,
    BLOCKED,
}
