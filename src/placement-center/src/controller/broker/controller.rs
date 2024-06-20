use crate::cache::placement::PlacementCacheManager;
use std::sync::{Arc, RwLock};

#[derive(Default, Clone)]
pub struct BrokerServerController {
    pub cluster_cache: Arc<RwLock<PlacementCacheManager>>,
}

impl BrokerServerController {
    pub fn new(cluster_cache: Arc<RwLock<PlacementCacheManager>>) -> BrokerServerController {
        return BrokerServerController { cluster_cache };
    }

    pub async fn start(&self) {}
}
