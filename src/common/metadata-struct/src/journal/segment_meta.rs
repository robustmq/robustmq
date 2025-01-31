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

/// Segment metadata in the placement center.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct JournalSegmentMetadata {
    pub cluster_name: String,
    pub namespace: String,
    pub shard_name: String,
    pub segment_seq: u32,
    pub start_offset: i64,
    pub end_offset: i64,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

impl JournalSegmentMetadata {
    pub fn name(&self) -> String {
        format!(
            "{},{},{},{}",
            self.cluster_name, self.namespace, self.shard_name, self.segment_seq
        )
    }
}
