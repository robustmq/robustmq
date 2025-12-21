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

use crate::storage::keys::{key_node, key_node_prefix};
use common_base::error::common::CommonError;
use metadata_struct::meta::node::BrokerNode;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};
use std::sync::Arc;

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
        let node_key = key_node(node.node_id);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &node_key, node.clone())
    }

    pub fn delete(&self, node_id: u64) -> Result<(), CommonError> {
        let node_key = key_node(node_id);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &node_key)
    }

    #[allow(dead_code)]
    pub fn get(&self, node_id: u64) -> Result<Option<BrokerNode>, CommonError> {
        let node_key = key_node(node_id);
        if let Some(data) =
            engine_get_by_meta_metadata::<BrokerNode>(&self.rocksdb_engine_handler, &node_key)?
        {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn list(&self) -> Result<Vec<BrokerNode>, CommonError> {
        let prefix_key = key_node_prefix();

        let data = engine_prefix_list_by_meta_metadata::<BrokerNode>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        let mut results = Vec::new();
        for raw in data {
            results.push(raw.data);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocksdb_engine::storage::family::column_family_list;
    use tempfile::tempdir;

    fn setup_kv_storage() -> NodeStorage {
        let temp_dir = tempdir().unwrap();
        let engine =
            RocksDBEngine::new(temp_dir.path().to_str().unwrap(), 100, column_family_list());
        NodeStorage::new(Arc::new(engine))
    }

    fn get_test_node() -> BrokerNode {
        BrokerNode {
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
        let test_node = kv.get(1).unwrap().unwrap();
        assert_eq!(test_node.node_inner_addr, node.node_inner_addr);
    }

    #[test]
    fn test_delete_existing() {
        let kv = setup_kv_storage();
        let node = get_test_node();
        kv.save(&node).unwrap();
        kv.delete(1).unwrap();
        let option = kv.get(1).unwrap();
        assert!(option.is_none());
    }

    #[test]
    fn test_delete_non_existent() {
        let kv = setup_kv_storage();
        kv.delete(1).unwrap();
    }
}
