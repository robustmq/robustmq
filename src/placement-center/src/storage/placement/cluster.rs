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
        engine_delete_by_cluster, engine_get_by_cluster, engine_list_by_cluster,
        engine_save_by_cluster,
    },
    keys::{key_cluster, key_cluster_prefix, key_cluster_prefix_by_type},
    rocksdb::RocksDBEngine,
};
use common_base::errors::RobustMQError;
use metadata_struct::placement::cluster::ClusterInfo;
use std::sync::Arc;

pub struct ClusterStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ClusterStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ClusterStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, cluster_info: ClusterInfo) -> Result<(), RobustMQError> {
        let key = key_cluster(&cluster_info.cluster_type, &cluster_info.cluster_name);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, cluster_info);
    }

    #[allow(dead_code)]
    pub fn get(
        &self,
        cluster_type: &String,
        cluster_name: &String,
    ) -> Result<Option<ClusterInfo>, RobustMQError> {
        let key = key_cluster(cluster_type, cluster_name);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<ClusterInfo>(&data.data) {
                Ok(info) => {
                    return Ok(Some(info));
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        }
    }

    #[allow(dead_code)]
    pub fn delete(
        &self,
        cluster_type: &String,
        cluster_name: &String,
    ) -> Result<(), RobustMQError> {
        let key: String = key_cluster(cluster_type, cluster_name);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }

    pub fn list(&self, cluster_type: Option<String>) -> Result<Vec<ClusterInfo>, RobustMQError> {
        let prefix_key = if let Some(ct) = cluster_type {
            key_cluster_prefix_by_type(&ct)
        } else {
            key_cluster_prefix()
        };
        match engine_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<ClusterInfo>(&raw.data) {
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
