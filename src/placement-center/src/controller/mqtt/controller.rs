use crate::cache::placement::PlacementCacheManager;
use std::sync::{Arc, RwLock};

use super::{
    retain_message_expire::start_retain_message_expire_check,
    session_expire::start_session_expire_check,
};

#[derive(Default, Clone)]
pub struct BrokerServerController {
    pub cluster_cache: Arc<RwLock<PlacementCacheManager>>,
}

impl BrokerServerController {
    pub fn new(cluster_cache: Arc<RwLock<PlacementCacheManager>>) -> BrokerServerController {
        return BrokerServerController { cluster_cache };
    }

    pub async fn start(&self) {
        start_retain_message_expire_check().await;
        start_session_expire_check().await;
    }
}
