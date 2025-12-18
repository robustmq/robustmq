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

use common_base::error::common::CommonError;
use metadata_struct::adapter::ShardInfo;

pub struct AdapterHandler {}

impl AdapterHandler {
    pub fn new() -> Self {
        AdapterHandler {}
    }

    pub async fn create_shard(&self, _shard: &ShardInfo) -> Result<(), CommonError> {
        Ok(())
    }

    pub async fn list_shard(&self, _shard: &str) -> Result<Vec<ShardInfo>, CommonError> {
        Ok(Vec::new())
    }

    pub async fn delete_shard(&self, _shard: &str) -> Result<(), CommonError> {
        Ok(())
    }
}
