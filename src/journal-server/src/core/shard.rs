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

use common_base::error::journal_server::JournalServerError;
use dashmap::DashMap;
use metadata_struct::journal::shard::JournalShard;

pub struct ShardManager {
    shards: DashMap<String, JournalShard>,
}

impl ShardManager {
    pub fn new() -> Self {
        let shards = DashMap::with_capacity(8);
        ShardManager { shards }
    }

    pub fn get_shard(&self, shar_name: &str) -> Option<JournalShard> {
        if let Some(shard) = self.shards.get(shar_name) {
            return Some(shard.clone());
        }
        None
    }

    pub fn create_shard(&self) -> Result<(), JournalServerError> {
        Ok(())
    }

    pub fn delete_shard(&self) -> Result<(), JournalServerError> {
        Ok(())
    }
}
