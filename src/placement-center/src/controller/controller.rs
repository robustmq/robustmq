use crate::{
    cache::cluster::ClusterCache,
    controller::heartbeat::StorageEngineNodeHeartBeat,
    raft::apply::RaftMachineApply,
    storage::{
        cluster::{cluster::ClusterStorage, node::NodeStorage},
        rocksdb::RocksDBEngine,
    },
};
use common_base::{config::placement_center::placement_center_conf, log::info_meta};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct ClusterController {
    cluster_cache: Arc<ClusterCache>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_send: broadcast::Sender<bool>,
}

impl ClusterController {
    pub fn new(
        cluster_cache: Arc<ClusterCache>,
        placement_center_storage: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        stop_send: broadcast::Sender<bool>,
    ) -> ClusterController {
        let controller = ClusterController {
            cluster_cache,
            placement_center_storage,
            rocksdb_engine_handler,
            stop_send,
        };
        controller.load_cache();
        return controller;
    }

    pub async fn start(&self) {
        self.start_node_heartbeat_check();
        self.controller_manager_thread();
        info_meta("Cluster Controller started successfully");
    }

    pub fn controller_manager_thread(&self) {}

    pub fn load_cache(&self) {
        let cluster_handler = ClusterStorage::new(self.rocksdb_engine_handler.clone());
        let cluster_list = cluster_handler.all_cluster();

        let node_handler = NodeStorage::new(self.rocksdb_engine_handler.clone());

        for cluster in cluster_list {
            let cluster_name = cluster.cluster_name.clone();

            // load cluster cache
            self.cluster_cache.add_cluster(cluster.clone());

            // load node cache
            let node_list = node_handler.node_list(cluster_name.clone());
            for node in node_list {
                self.cluster_cache.add_node(node);
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
}
