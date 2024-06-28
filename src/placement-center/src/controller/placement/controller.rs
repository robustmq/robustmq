use super::heartbeat::BrokerHeartbeat;
use crate::{
    cache::placement::PlacementCacheManager, controller::mqtt::MQTTController,
    raft::apply::RaftMachineApply, storage::rocksdb::RocksDBEngine,
};
use common_base::{config::placement_center::placement_center_conf, log::info_meta};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct ClusterController {
    cluster_cache: Arc<PlacementCacheManager>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_send: broadcast::Sender<bool>,
}

impl ClusterController {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
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
        return controller;
    }

    pub fn start(&self) {
        self.start_node_heartbeat_check();
        info_meta("Cluster Controller started successfully");
    }

    // Start the heartbeat detection thread of the Storage Engine node
    pub fn start_node_heartbeat_check(&self) {
        let stop_recv = self.stop_send.subscribe();
        let config = placement_center_conf();
        let mut heartbeat = BrokerHeartbeat::new(
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
