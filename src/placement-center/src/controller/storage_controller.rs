use crate::{storage_cluster::StorageCluster, storage::cluster_storage::{ClusterInfo, NodeInfo, ShardInfo}};
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub struct StorageEngineController {
   pub storage_cluser: Arc<StorageCluster>,
}

impl StorageEngineController {
    pub fn new(broker_cluser: Arc<StorageCluster>) -> StorageEngineController {
        let mut controller = StorageEngineController::default();
        controller.storage_cluser = broker_cluser;
        return controller;
    }

    pub async fn start(&self){
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
