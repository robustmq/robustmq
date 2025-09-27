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
use metadata_struct::placement::node::BrokerNode;

use crate::storage::engine_meta::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_meta,
};
use crate::storage::keys::{key_node, key_node_prefix, key_node_prefix_all};
use rocksdb_engine::RocksDBEngine;

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
        engine_save_by_meta(self.rocksdb_engine_handler.clone(), node_key, node.clone())
    }

    pub fn delete(&self, cluster_name: &str, node_id: u64) -> Result<(), CommonError> {
        let node_key = key_node(cluster_name, node_id);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), node_key)
    }

    #[allow(dead_code)]
    pub fn get(&self, cluster_name: &str, node_id: u64) -> Result<Option<BrokerNode>, CommonError> {
        let node_key = key_node(cluster_name, node_id);
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), node_key)? {
            return Ok(Some(serde_json::from_str::<BrokerNode>(&data.data)?));
        }
        Ok(None)
    }

    pub fn list(&self, cluster_name: Option<String>) -> Result<Vec<BrokerNode>, CommonError> {
        let prefix_key = if let Some(cn) = cluster_name {
            key_node_prefix(&cn)
        } else {
            key_node_prefix_all()
        };

        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<BrokerNode>(&raw.data)?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use broker_core::rocksdb::column_family_list;
    use tempfile::tempdir;

    fn setup_kv_storage() -> NodeStorage {
        let temp_dir = tempdir().unwrap();
        let engine =
            RocksDBEngine::new(temp_dir.path().to_str().unwrap(), 100, column_family_list());
        NodeStorage::new(Arc::new(engine))
    }

    fn get_test_node() -> BrokerNode {
        BrokerNode {
            cluster_name: "test_cluster".to_string(),
            node_id: 1,
            node_ip: "127.0.0.1".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_save_and_get() {
        let kv = setup_kv_storage();
        let node = get_test_node();
        kv.save(&node).unwrap();
        let test_node = kv.get("test_cluster", 1).unwrap().unwrap();
        assert_eq!(test_node.cluster_name, node.cluster_name);
    }

    #[test]
    fn test_delete_existing() {
        let kv = setup_kv_storage();
        let node = get_test_node();
        kv.save(&node).unwrap();
        kv.delete("test_cluster", 1).unwrap();
        let option = kv.get("test_cluster", 1).unwrap();
        assert!(option.is_none());
    }

    #[test]
    fn test_delete_non_existent() {
        let kv = setup_kv_storage();
        kv.delete("test_cluster", 1).unwrap();
    }
}
