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

use common_base::{
    error::common::CommonError,
    tools::{now_second, unique_id},
    utils::serialize,
};
use common_config::storage::StorageType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]

pub struct EngineShard {
    pub shard_uid: String,
    pub shard_name: String,
    pub start_segment_seq: u32,
    pub active_segment_seq: u32,
    pub last_segment_seq: u32,
    pub status: EngineShardStatus,
    pub config: EngineShardConfig,
    pub create_time: u64,
}

impl EngineShard {
    pub fn new(shard_name: String, config: EngineShardConfig) -> Self {
        EngineShard {
            shard_uid: unique_id(),
            shard_name,
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            status: EngineShardStatus::Run,
            config,
            create_time: now_second(),
        }
    }
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EngineShardStatus {
    #[default]
    Run,
    PrepareDelete,
    Deleting,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EngineShardConfig {
    pub replica_num: u32,
    pub storage_type: StorageType,
    pub max_segment_size: u64,
    pub retention_sec: u64,
}

impl Default for EngineShardConfig {
    fn default() -> Self {
        Self {
            replica_num: 1,
            max_segment_size: 1073741824,
            retention_sec: 86400,
            storage_type: StorageType::EngineMemory,
        }
    }
}

impl EngineShardConfig {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}
