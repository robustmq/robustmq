use crate::cache::broker_cluster::BrokerClusterCache;
use std::sync::{Arc, RwLock};

#[derive(Default, Clone)]
pub struct BrokerServerController {
    pub broker_cache: Arc<RwLock<BrokerClusterCache>>,
}

impl BrokerServerController {
    pub fn new(broker_cache: Arc<RwLock<BrokerClusterCache>>) -> BrokerServerController {
        let mut bsc = BrokerServerController::default();
        bsc.broker_cache = broker_cache;
        return bsc;
    }

    pub async fn start(&self) {}
}
