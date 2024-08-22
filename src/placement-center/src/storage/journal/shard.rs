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

use crate::storage::{
    engine::{
        engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
        engine_save_by_cluster,
    },
    keys::{key_shard, key_shard_prefix},
    rocksdb::RocksDBEngine,
};

use common_base::error::robustmq::RobustMQError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_uid: String,
    pub cluster_name: String,
    pub shard_name: String,
    pub replica: u32,
    pub last_segment_seq: u64,
    pub create_time: u128,
}

pub struct ShardStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShardStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ShardStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, shard_info: ShardInfo) -> Result<(), RobustMQError> {
        let shard_key = key_shard(&shard_info.cluster_name, &shard_info.shard_name);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), shard_key, shard_info);
    }

    pub fn get(
        &self,
        cluster_name: &String,
        shard_name: &String,
    ) -> Result<Option<ShardInfo>, RobustMQError> {
        let shard_key: String = key_shard(&cluster_name, shard_name);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), shard_key) {
            Ok(Some(data)) => match serde_json::from_slice::<ShardInfo>(&data.data) {
                Ok(shard) => {
                    return Ok(Some(shard));
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => Err(e),
        }
    }

    pub fn delete(&self, cluster_name: &String, shard_name: &String) -> Result<(), RobustMQError> {
        let shard_key = key_shard(&cluster_name, shard_name);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), shard_key);
    }

    pub fn list_by_shard(&self, cluster_name: &String) -> Result<Vec<ShardInfo>, RobustMQError> {
        let prefix_key = key_shard_prefix(cluster_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<ShardInfo>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
