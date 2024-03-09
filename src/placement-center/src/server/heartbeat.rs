use common::log::error_meta;
use protocol::placement_center::placement::{ClusterType, UnRegisterNodeRequest};
use std::{
    sync::{Arc, RwLock},
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    cache::{broker_cluster::BrokerClusterCache, engine_cluster::EngineClusterCache},
    raft::storage::PlacementCenterStorage,
};

#[derive(Clone)]
pub struct Heartbeat {
    timeout_ms: u128,
    check_time_ms: u128,
    engine_cache: Arc<RwLock<EngineClusterCache>>,
    broker_cluster: Arc<RwLock<BrokerClusterCache>>,
    placement_center_storage: Arc<PlacementCenterStorage>,
}

impl Heartbeat {
    pub fn new(
        timeout_ms: u128,
        check_time_ms: u128,
        engine_cache: Arc<RwLock<EngineClusterCache>>,
        broker_cluster: Arc<RwLock<BrokerClusterCache>>,
        placement_center_storage: Arc<PlacementCenterStorage>,
    ) -> Self {
        return Heartbeat {
            timeout_ms,
            check_time_ms,
            engine_cache,
            broker_cluster,
            placement_center_storage,
        };
    }

    pub async fn start_heartbeat_check(&self) {
        loop {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            // storage engine cluster
            let ec = self.engine_cache.read().unwrap();
            for (node_id, node) in &ec.node_list {
                if let Some(prev_time) = ec.node_heartbeat.get(node_id) {
                    if time - *prev_time >= self.timeout_ms {
                        // remove node
                        let cluster_name = node.cluster_name.clone();
                        if let Some(_) = ec.cluster_list.get(&cluster_name) {
                            let mut req = UnRegisterNodeRequest::default();
                            req.node_id = *node_id;
                            req.cluster_name = node.cluster_name.clone();
                            req.cluster_type = ClusterType::StorageEngine.into();
                            let pcs = self.placement_center_storage.clone();
                            tokio::spawn(async move {
                                match pcs.delete_node(req).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error_meta(&e.to_string());
                                    }
                                }
                            });
                        }
                    }
                } else {
                    let mut ec = self.engine_cache.write().unwrap();
                    ec.heart_time(*node_id, time);
                }
            }

            // broker server cluster

            sleep(Duration::from_millis(100));
        }
    }
}
