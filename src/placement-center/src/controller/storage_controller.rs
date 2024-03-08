use crate::{
    cache::engine_cluster::EngineClusterCache,
    rocksdb::data_rw_layer::{ClusterInfo, NodeInfo, ShardInfo},
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default)]
pub struct StorageEngineController {
    pub engine_cache: Arc<RwLock<EngineClusterCache>>,
}

impl StorageEngineController {
    pub fn new(engine_cache: Arc<RwLock<EngineClusterCache>>) -> StorageEngineController {
        let mut controller = StorageEngineController::default();
        controller.engine_cache = engine_cache;
        return controller;
    }

    pub async fn start(&self) {
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
