// Copyright 2023 RobustMQ Team
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


use crate::{
    record::Record,
    storage::{ShardConfig, StorageAdapter},
};
use axum::async_trait;
use clients::{
    placement::kv::call::{placement_delete, placement_exists, placement_get, placement_set},
    poll::ClientPool,
};
use common_base::error::robustmq::RobustMQError;
use protocol::placement_center::generate::kv::{
    DeleteRequest, ExistsRequest, GetRequest, SetRequest,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PlacementStorageAdapter {
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
}

impl PlacementStorageAdapter {
    pub fn new(client_poll: Arc<ClientPool>, addrs: Vec<String>) -> Self {
        return PlacementStorageAdapter { client_poll, addrs };
    }
}

#[async_trait]
impl StorageAdapter for PlacementStorageAdapter {
    async fn create_shard(
        &self,
        shard_name: String,
        shard_config: ShardConfig,
    ) -> Result<(), RobustMQError> {
        return Ok(());
    }

    async fn delete_shard(&self, shard_name: String) -> Result<(), RobustMQError> {
        return Ok(());
    }

    async fn set(&self, key: String, value: Record) -> Result<(), RobustMQError> {
        let request = SetRequest {
            key,
            value: String::from_utf8(value.data).unwrap(),
        };
        match placement_set(self.client_poll.clone(), self.addrs.clone(), request).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    async fn get(&self, key: String) -> Result<Option<Record>, RobustMQError> {
        let request = GetRequest { key };
        match placement_get(self.client_poll.clone(), self.addrs.clone(), request).await {
            Ok(reply) => {
                if reply.value.is_empty() {
                    return Ok(None);
                } else {
                    return Ok(Some(Record::build_e(reply.value)));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    async fn delete(&self, key: String) -> Result<(), RobustMQError> {
        let request = DeleteRequest { key };
        match placement_delete(self.client_poll.clone(), self.addrs.clone(), request).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                return Err(e);
            }
        }
    }
    async fn exists(&self, key: String) -> Result<bool, RobustMQError> {
        let request = ExistsRequest { key };
        match placement_exists(self.client_poll.clone(), self.addrs.clone(), request).await {
            Ok(reply) => return Ok(reply.flag),
            Err(e) => {
                return Err(e);
            }
        }
    }

    async fn stream_write(&self, _: String, _: Vec<Record>) -> Result<Vec<usize>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read(
        &self,
        _: String,
        _: String,
        _: Option<u128>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_commit_offset(
        &self,
        _: String,
        _: String,
        _: u128,
    ) -> Result<bool, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_offset(
        &self,
        _: String,
        _: usize,
    ) -> Result<Option<Record>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_timestamp(
        &self,
        _: String,
        _: u128,
        _: u128,
        _: Option<usize>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_key(
        &self,
        _: String,
        _: String,
    ) -> Result<Option<Record>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }
}
