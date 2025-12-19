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

use std::sync::Arc;

use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record, ShardInfo, ShardOffset};

use crate::core::{
    cache::StorageCacheManager,
    shard::{create_shard_to_place, delete_shard_to_place},
};

#[derive(Clone)]
pub struct AdapterHandler {
    cache_manager: Arc<StorageCacheManager>,
    client_pool: Arc<ClientPool>,
}

impl AdapterHandler {
    pub fn new(cache_manager: Arc<StorageCacheManager>, client_pool: Arc<ClientPool>) -> Self {
        AdapterHandler {
            cache_manager,
            client_pool,
        }
    }

    pub async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        if let Err(e) = create_shard_to_place(&self.cache_manager, &self.client_pool, shard).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    pub async fn list_shard(&self, shard: Option<String>) -> Result<Vec<ShardInfo>, CommonError> {
        if let Some(shard_name) = shard {
            if let Some(raw) = self.cache_manager.shards.get(&shard_name) {
                return Ok(vec![ShardInfo {
                    shard_name: raw.shard_name.clone(),
                    replica_num: 1,
                }]);
            }
            return Ok(Vec::new());
        }

        let res = self
            .cache_manager
            .shards
            .iter()
            .map(|raw| ShardInfo {
                shard_name: raw.shard_name.clone(),
                replica_num: 1,
            })
            .collect();

        Ok(res)
    }

    pub async fn delete_shard(&self, shard_name: &str) -> Result<(), CommonError> {
        if let Err(e) = delete_shard_to_place(&self.client_pool, shard_name).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    pub async fn batch_write(
        &self,
        shard: &str,
        records: &[Record],
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
