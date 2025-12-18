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
use metadata_struct::adapter::{read_config::ReadConfig, record::Record, ShardInfo, ShardOffset};

#[derive(Default, Clone)]
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

    pub async fn batch_write(
        &self,
        _shard: &str,
        _records: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        Ok(Vec::new())
    }

    pub async fn read_by_offset(
        &self,
        _shard: &str,
        _offset: u64,
        _read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    pub async fn read_by_tag(
        &self,
        _shard: &str,
        _tag: &str,
        _start_offset: Option<u64>,
        _read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    pub async fn read_by_key(&self, _shard: &str, _key: &str) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    pub async fn get_offset_by_timestamp(
        &self,
        _shard: &str,
        _timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        Ok(None)
    }
}
