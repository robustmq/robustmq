use crate::storage::cluster_storage::{ClusterInfo, NodeInfo, ShardInfo};
use std::collections::HashMap;

#[derive(Default)]
pub struct StorageEngineController {
    pub cluster_list: HashMap<String, ClusterInfo>,
    pub node_list: HashMap<u64, NodeInfo>,
    pub shard_list: HashMap<String, ShardInfo>,
}

impl StorageEngineController {
    pub fn new() -> StorageEngineController {
        let mut controller = StorageEngineController::default();
        controller.cluster_list = controller.load_cluster_list();
        controller.node_list = controller.load_node_list();
        controller.shard_list = controller.load_shard_list();
        return controller;
    }

    pub fn start(&self){
        // 
    }

    fn load_cluster_list(&self) -> HashMap<String, ClusterInfo> {
        return HashMap::default();
    }

    fn load_node_list(&self) -> HashMap<u64, NodeInfo> {
        return HashMap::default();
    }

    fn load_shard_list(&self) -> HashMap<String, ShardInfo> {
        return HashMap::default();
    }
}
