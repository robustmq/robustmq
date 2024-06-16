use super::preferred_election::PreferredElection;
use crate::{
    cache::journal::JournalCache,
    raft::apply::RaftMachineApply, storage::{cluster::{cluster::ClusterStorage, node::NodeStorage}, journal::shard::ShardStorage, rocksdb::RocksDBEngine},
};
use common_base::log::info_meta;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

pub struct StorageEngineController {
    engine_cache: Arc<RwLock<JournalCache>>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_send: broadcast::Sender<bool>,
}

impl StorageEngineController {
    pub fn new(
        engine_cache: Arc<RwLock<JournalCache>>,
        placement_center_storage: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        stop_send: broadcast::Sender<bool>,
    ) -> StorageEngineController {
        let controller = StorageEngineController {
            engine_cache,
            placement_center_storage,
            rocksdb_engine_handler,
            stop_send,
        };
        controller.load_cache();
        return controller;
    }

    pub async fn start(&self) {
        self.resource_manager_thread();
        self.preferred_replica_election();
        info_meta("Storage Engine Controller started successfully");
    }

    pub fn load_cache(&self) {
        let cluster_handler = ClusterStorage::new(self.rocksdb_engine_handler.clone());
        let cluster_list = cluster_handler.all_cluster();

        let mut engine_cache = self.engine_cache.write().unwrap();
        let node_handler = NodeStorage::new(self.rocksdb_engine_handler.clone());
        let shard_handler = ShardStorage::new(self.rocksdb_engine_handler.clone());

        for cluster in cluster_list {
            let cluster_name = cluster.cluster_name.clone();

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
