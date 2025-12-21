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

use metadata_struct::storage::segment::{segment_name, EngineSegment};

pub mod file;
pub mod index;
pub mod isr;
pub mod keys;
pub mod manager;
pub mod read;
pub mod scroll;
// pub mod write;
pub mod write0;

/// A unique identifier for a segment, used to get segment metadata or segment file.
#[derive(Clone, Debug)]
pub struct SegmentIdentity {
    pub shard_name: String,
    pub segment: u32,
}

impl SegmentIdentity {
    pub fn name(&self) -> String {
        segment_name(&self.shard_name, self.segment)
    }

    pub fn new(shard_name: &str, segment_seq: u32) -> Self {
        SegmentIdentity {
            shard_name: shard_name.to_string(),
            segment: segment_seq,
        }
    }

    pub fn from_journal_segment(segment: &EngineSegment) -> Self {
        SegmentIdentity {
            shard_name: segment.shard_name.to_string(),
            segment: segment.segment_seq,
        }
    }
}
