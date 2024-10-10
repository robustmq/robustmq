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
use metadata_struct::placement::broker_node::BrokerNode;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{key_node, key_node_prefix, key_node_prefix_all};
use crate::storage::rocksdb::RocksDBEngine;

pub struct NodeStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl NodeStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        NodeStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, node: &BrokerNode) -> Result<(), CommonError> {
        let node_key = key_node(&node.cluster_name, node.node_id);
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), node_key, node.clone())
    }

    pub fn delete(&self, cluster_name: &String, node_id: u64) -> Result<(), CommonError> {
        let node_key = key_node(cluster_name, node_id);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), node_key)
    }

    #[allow(dead_code)]
    pub fn get(
        &self,
        cluster_name: &String,
        node_id: u64,
    ) -> Result<Option<BrokerNode>, CommonError> {
        let node_key = key_node(cluster_name, node_id);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), node_key) {
            Ok(Some(data)) => match serde_json::from_slice::<BrokerNode>(&data.data) {
                Ok(node) => Ok(Some(node)),
                Err(e) => Err(e.into()),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn list(&self, cluster_name: Option<String>) -> Result<Vec<BrokerNode>, CommonError> {
        let prefix_key = if let Some(cn) = cluster_name {
            key_node_prefix(&cn)
        } else {
            key_node_prefix_all()
        };

        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<BrokerNode>(&raw.data) {
                        Ok(node) => {
                            results.push(node);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }
}
