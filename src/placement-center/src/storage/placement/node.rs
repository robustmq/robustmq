use crate::storage::{
    keys::{key_node, key_node_prefix},
    rocksdb::RocksDBEngine,
};
use common_base::errors::RobustMQError;
use metadata_struct::placement::broker_node::BrokerNode;
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

    pub fn save(
        &self,
        cluster_name: String,
        node: BrokerNode,
    ) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let node_key = key_node(cluster_name, node.node_id);
        match self.rocksdb_engine_handler.write(cf, &node_key, &node) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn delete(&self, cluster_name: String, node_id: u64) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let node_key = key_node(cluster_name, node_id);
        match self.rocksdb_engine_handler.delete(cf, &node_key) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn get(
        &self,
        cluster_name: String,
        node_id: u64,
    ) -> Result<Option<BrokerNode>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let cluster_key = key_node(cluster_name, node_id);
        match self
            .rocksdb_engine_handler
            .read::<BrokerNode>(cf, &cluster_key)
        {
            Ok(cluster_info) => {
                return Ok(cluster_info);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn list(&self, cluster_name: String) -> Result<Vec<BrokerNode>, RobustMQError> {
        let mut result = Vec::new();
        let prefix_key = key_node_prefix(cluster_name);
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let data_list = self.rocksdb_engine_handler.read_prefix(cf, &prefix_key);
        let mut results = Vec::new();
        for raw in data_list {
            for (_, v) in raw {
                match serde_json::from_slice::<BrokerNode>(v.as_ref()) {
                    Ok(v) => results.push(v),
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
        return Ok(result);
    }
}
