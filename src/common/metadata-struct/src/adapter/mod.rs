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

pub mod read_config;
pub mod record;
pub mod serde_bytes_wrapper;

pub enum OffsetStrategy {
    Earliest,
    Latest,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_name: String,
    pub replica_num: u32,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct ShardOffset {
    pub group: String,
    pub shard_name: String,
    pub segment_no: u32,
    pub offset: u64,
}

#[derive(Default, Clone)]
pub struct MessageExpireConfig {
    // data_size: Option<u32>,
    timestamp: Option<u32>,
}

impl MessageExpireConfig {
    /// Get the retention period in seconds
    pub fn get_timestamp(&self) -> Option<u32> {
        self.timestamp
    }

    /// Set the retention period in seconds
    pub fn set_timestamp(&mut self, timestamp: Option<u32>) {
        self.timestamp = timestamp;
    }

    /// Create a new config with timestamp retention
    pub fn with_timestamp(timestamp: u32) -> Self {
        Self {
            timestamp: Some(timestamp),
        }
    }
}

#[derive(Default, Clone)]
pub enum MessageExpireStrategy {
    #[default]
    // DataSize,
    Timestamp,
}
