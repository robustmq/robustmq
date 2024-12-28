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

#[derive(Clone, Default)]
pub struct JournalEngineClusterConfig {
    pub enable_auto_create_shard: bool,
    pub default_shard_replica_num: u32,
    pub last_update_local_cache_time: u64,
}

impl JournalEngineClusterConfig {
    pub fn new() -> Self {
        JournalEngineClusterConfig {
            enable_auto_create_shard: true,
            default_shard_replica_num: 2,
            last_update_local_cache_time: 0,
        }
    }
}
