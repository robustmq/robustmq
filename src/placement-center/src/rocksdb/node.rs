use super::{
    cluster::{ClusterInfo, ClusterStorage},
    keys::key_node,
    rocksdb::RocksDBEngine,
};
use common::{config::placement_center::placement_center_conf, log::error_meta};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: u64,
    pub node_ip: String,
    pub node_port: u32,
    pub cluster_name: String,
}

pub struct NodeStorage {
    rocksdb_engine: RocksDBEngine,
    cluster_storage: ClusterStorage,
}

impl NodeStorage {
    pub fn new() -> Self {
        let config = placement_center_conf();
        let rocksdb_engine = RocksDBEngine::new(&config);
        let cluster_storage = ClusterStorage::new();
        NodeStorage {
            rocksdb_engine,
            cluster_storage,
        }
    }

    pub fn node_list(&self, cluster_name: String) -> Vec<NodeInfo> {
        let mut result = Vec::new();
        let cf = self.rocksdb_engine.cf_cluster();

        let cluster_info = self.cluster_storage.get_cluster(&cluster_name);
        if cluster_info.is_none() {
            return result;
        }

        for node_id in cluster_info.unwrap().nodes {
            let key = key_node(&cluster_name, node_id);
            match self.rocksdb_engine.read::<NodeInfo>(cf, &key) {
                Ok(node_info) => {
                    if let Some(ni) = node_info {
                        result.push(ni);
                    }
                }
                Err(_) => {}
            };
        }
        return result;
    }

    // save node info
    pub fn save_node(&self, cluster_name: String, cluster_type: String, node: NodeInfo) {
        let cf = self.rocksdb_engine.cf_cluster();

        // save or update cluster info
        let mut ci = ClusterInfo::default();
        let mut cluster_info = self.cluster_storage.get_cluster(&cluster_name);
        if cluster_info.is_none() {
            ci.cluster_name = cluster_name.clone();
        } else {
            ci = cluster_info.unwrap();
        }
        ci.nodes.push(node.node_id);
        ci.nodes.dedup();
        self.cluster_storage.save_cluster(ci);

        // save node info
        let node_key = key_node(&cluster_name, node.node_id);
        match self.rocksdb_engine.write(cf, &node_key, &node) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn remove_node(&self, cluster_name: String, node_id: u64) {
        let cf = self.rocksdb_engine.cf_cluster();

        // save or update cluster info
        let mut cluster_info = self.cluster_storage.get_cluster(&cluster_name);
        if !cluster_info.is_none() {
            let mut ci = cluster_info.unwrap();
            let mut nodes = Vec::new();
            for nid in ci.nodes {
                if nid != node_id {
                    nodes.push(nid);
                }
            }
            ci.nodes = nodes;
            self.cluster_storage.save_cluster(ci);
        }

        // delete node info
        let node_key = key_node(&cluster_name, node_id);
        match self.rocksdb_engine.delete(cf, &node_key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    // get node info
    pub fn get_node(&self, cluster_name: String, node_id: u64) -> Option<NodeInfo> {
        let cf = self.rocksdb_engine.cf_cluster();
        let cluster_key = key_node(&cluster_name, node_id);
        match self.rocksdb_engine.read::<NodeInfo>(cf, &cluster_key) {
            Ok(cluster_info) => {
                return cluster_info;
            }
            Err(_) => {}
        }
        return None;
    }
}
