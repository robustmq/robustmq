use crate::{
    cache::cluster::{node_key, ClusterCache},
    raft::storage::PlacementCenterStorage,
};
use common_base::log::{error_meta, info_meta};
use protocol::placement_center::generate::{common::ClusterType, placement::UnRegisterNodeRequest};
use std::{
    sync::Arc,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::broadcast;

pub struct StorageEngineNodeHeartBeat {
    timeout_ms: u128,
    check_time_ms: u64,
    cluster_cache: Arc<ClusterCache>,
    placement_center_storage: Arc<PlacementCenterStorage>,
    stop_recv: broadcast::Receiver<bool>,
}

impl StorageEngineNodeHeartBeat {
    pub fn new(
        timeout_ms: u128,
        check_time_ms: u64,
        cluster_cache: Arc<ClusterCache>,
        placement_center_storage: Arc<PlacementCenterStorage>,
        stop_recv: broadcast::Receiver<bool>,
    ) -> Self {
        return StorageEngineNodeHeartBeat {
            timeout_ms,
            check_time_ms,
            cluster_cache,
            placement_center_storage,
            stop_recv,
        };
    }

    pub async fn start(&mut self) {
        info_meta("Storage Engine Cluster node heartbeat detection thread started successfully");
        loop {
            match self.stop_recv.try_recv() {
                Ok(val) => {
                    if val {
                        break;
                    }
                }
                Err(_) => {}
            }

            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            for (_, node) in self.cluster_cache.node_list.clone() {
                let key = node_key(node.cluster_name.clone(), node.node_id);
                if let Some(prev_time) = self.cluster_cache.node_heartbeat.get(&key) {
                    // Detects whether the heartbeat rate of a node exceeds the unreported rate.
                    // If yes, remove the node
                    if time - *prev_time >= self.timeout_ms {
                        let cluster_name = node.cluster_name.clone();
                        if let Some(_) = self.cluster_cache.cluster_list.get(&cluster_name) {
                            let mut req = UnRegisterNodeRequest::default();
                            req.node_id = node.node_id;
                            req.cluster_name = node.cluster_name.clone();
                            req.cluster_type = ClusterType::StorageEngine.into();
                            let pcs = self.placement_center_storage.clone();

                            tokio::spawn(async move {
                                match pcs.delete_node(req).await {
                                    Ok(_) => {
                                        info_meta(
                                            &format!("The heartbeat of the Storage Engine node times out and is deleted from the cluster. Node ID: {}, node IP: {}.",
                                            node.node_id,
                                            node.node_ip));
                                    }
                                    Err(e) => {
                                        error_meta(&e.to_string());
                                    }
                                }
                            });
                        }
                    }
                } else {
                    self.cluster_cache.heart_time(key, time);
                }
            }
            sleep(Duration::from_millis(self.check_time_ms));
        }
    }
}
