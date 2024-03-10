use super::engine_heartbeat::StorageEngineNodeHeartBeat;
use crate::{
    cache::engine_cluster::EngineClusterCache,
    raft::storage::PlacementCenterStorage,
    rocksdb::{cluster::ClusterInfo, node::NodeInfo, shard::ShardInfo},
};
use common::log::info_meta;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast;

pub struct StorageEngineController {
    engine_cache: Arc<RwLock<EngineClusterCache>>,
    placement_center_storage: Arc<PlacementCenterStorage>,
    stop_send: broadcast::Sender<bool>,
}

impl StorageEngineController {
    pub fn new(
        engine_cache: Arc<RwLock<EngineClusterCache>>,
        placement_center_storage: Arc<PlacementCenterStorage>,
        stop_send: broadcast::Sender<bool>,
    ) -> StorageEngineController {
        return StorageEngineController {
            engine_cache,
            placement_center_storage,
            stop_send,
        };
    }

    pub async fn start(&self) {
        self.start_node_heartbeat_check();
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

    // Start the heartbeat detection thread of the Storage Engine node
    pub fn start_node_heartbeat_check(&self) {
        let stop_recv = self.stop_send.subscribe();
        let mut heartbeat = StorageEngineNodeHeartBeat::new(
            100000,
            1000,
            self.engine_cache.clone(),
            self.placement_center_storage.clone(),
            stop_recv,
        );
        tokio::spawn(async move {
            heartbeat.start().await;
        });
        info_meta("Storage Engine ßCluster node heartbeat detection thread started successfully");
    }
}
