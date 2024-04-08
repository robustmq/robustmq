use crate::cache::cluster::ClusterCache;
use std::sync::{Arc, RwLock};

#[derive(Default, Clone)]
pub struct BrokerServerController {
    pub cluster_cache: Arc<RwLock<ClusterCache>>,
}

impl BrokerServerController {
    pub fn new(cluster_cache: Arc<RwLock<ClusterCache>>) -> BrokerServerController {
        return BrokerServerController { cluster_cache };
    }

    pub async fn start(&self) {}
}
