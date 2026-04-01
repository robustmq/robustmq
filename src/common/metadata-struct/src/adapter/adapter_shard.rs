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

use crate::storage::shard::{EngineShard, EngineShardConfig};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AdapterShardInfo {
    pub shard_name: String,
    pub config: EngineShardConfig,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterShardDetail {
    pub shard_name: String,
    pub config: EngineShardConfig,
    pub extend: AdapterShardDetailExtend,
    pub offset: AdapterShardDetailOffset,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterShardDetailExtend {
    StorageEngine(EngineShard),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterShardDetailOffset {
    pub start_offset: u64,
    pub end_offset: u64,
}
