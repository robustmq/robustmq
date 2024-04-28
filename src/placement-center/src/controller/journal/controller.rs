use super::preferred_election::PreferredElection;
use crate::{
    cache::{cluster::ClusterCache, engine::EngineCache}, controller::heartbeat::StorageEngineNodeHeartBeat, raft::storage::PlacementCenterStorage, storage::{
        cluster::ClusterStorage, node::NodeStorage, rocksdb::RocksDBEngine, shard::ShardStorage,
    }
};
use common_base::{config::placement_center::placement_center_conf, log::info_meta};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

pub struct StorageEngineController {
    cluster_cache: Arc<RwLock<ClusterCache>>,
    engine_cache: Arc<RwLock<EngineCache>>,
    placement_center_storage: Arc<PlacementCenterStorage>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_send: broadcast::Sender<bool>,
}

impl StorageEngineController {
    pub fn new(
        cluster_cache: Arc<RwLock<ClusterCache>>,
        engine_cache: Arc<RwLock<EngineCache>>,
        placement_center_storage: Arc<PlacementCenterStorage>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        stop_send: broadcast::Sender<bool>,
    ) -> StorageEngineController {
        let controller = StorageEngineController {
            cluster_cache,
            engine_cache,
            placement_center_storage,
            rocksdb_engine_handler,
            stop_send,
        };
        controller.load_cache();
        return controller;
    }

    pub async fn start(&self) {
        self.start_node_heartbeat_check();
        self.resource_manager_thread();
        self.preferred_replica_election();
        info_meta("Storage Engine Controller started successfully");
    }

    pub fn load_cache(&self) {
        let cluster_handler = ClusterStorage::new(self.rocksdb_engine_handler.clone());
        let cluster_list = cluster_handler.all_cluster();

        let mut engine_cache = self.engine_cache.write().unwrap();
        let mut cluster_cache = self.cluster_cache.write().unwrap();
        let node_handler = NodeStorage::new(self.rocksdb_engine_handler.clone());
        let shard_handler = ShardStorage::new(self.rocksdb_engine_handler.clone());

        for cluster in cluster_list {
            let cluster_name = cluster.cluster_name.clone();

            // load cluster cache
            cluster_cache.add_cluster(cluster.clone());

            // load node cache
            let node_list = node_handler.node_list(cluster_name.clone());
            for node in node_list {
                cluster_cache.add_node(node);
            }

            // load shard cache
            let shard_list = shard_handler.shard_list(cluster_name.clone());
            for shard in shard_list {
                engine_cache.add_shard(shard.clone());
                let segment_list =
                    shard_handler.segment_list(cluster_name.clone(), shard.shard_name);
                for segment in segment_list {
                    engine_cache.add_segment(segment);
                }
            }
        }
    }

    // Start the heartbeat detection thread of the Storage Engine node
    pub fn start_node_heartbeat_check(&self) {
        let stop_recv = self.stop_send.subscribe();
        let config = placement_center_conf();
        let mut heartbeat = StorageEngineNodeHeartBeat::new(
            config.heartbeat_timeout_ms.into(),
            config.heartbeat_check_time_ms,
            self.cluster_cache.clone(),
            self.placement_center_storage.clone(),
            stop_recv,
        );
        tokio::spawn(async move {
            heartbeat.start().await;
        });
    }

    pub fn resource_manager_thread(&self) {
        tokio::spawn(async move {});
    }

    pub fn preferred_replica_election(&self) {
        let election = PreferredElection::new(self.engine_cache.clone());
        tokio::spawn(async move {
            election.start().await;
        });
    }
}
