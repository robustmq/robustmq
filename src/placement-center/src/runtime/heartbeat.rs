use std::{
    sync::{Arc, RwLock},
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use common::log::error_meta;
use protocol::placement_center::placement::{ClusterType, UnRegisterNodeRequest};

use crate::{broker_cluster::BrokerCluster, client::unregister_node, storage_cluster::StorageCluster};

#[derive(Debug, Clone)]
pub struct Heartbeat {
    timeout_ms: u128,
    storage_cluster: Arc<RwLock<StorageCluster>>,
    broker_cluster: Arc<RwLock<BrokerCluster>>,
}

impl Heartbeat {
    pub fn new(
        timeout_ms: u128,
        storage_cluster: Arc<RwLock<StorageCluster>>,
        broker_cluster: Arc<RwLock<BrokerCluster>>,
    ) -> Self {
        return Heartbeat {
            timeout_ms,
            storage_cluster,
            broker_cluster,
        };
    }

    pub async fn start_heartbeat_check(&self) {
        loop {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            
            // storage engine cluster
            let mut ec = self.storage_cluster.read().unwrap();
            for (node_id, node) in &ec.node_list {
                if let Some(prev_time) = ec.node_heartbeat.get(node_id) {
                    if time - *prev_time >= self.timeout_ms {
                        // remove node
                        let cluster_name = node.cluster_name.clone();
                        if let Some(cluster) = ec.cluster_list.get(&cluster_name){
                            let mut req  = UnRegisterNodeRequest::default();
                            req.node_id = *node_id;
                            req.cluster_name = node.cluster_name.clone();
                            req.cluster_type = 1; // todo

                            let addr = "".to_string();
                            // match unregister_node(&addr, req).await {
                            //     Ok(_) => {},
                            //     Err(e) => error_meta(&e.to_string()),
                            // }
                        }
                    }
                } else {
                    //
                    self.storage_cluster
                        .write()
                        .unwrap()
                        .heart_time(*node_id, time);
                }
            }

             // broker server cluster

            sleep(Duration::from_millis(100));
        }
    }
}
