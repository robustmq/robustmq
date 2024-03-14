use std::sync::{Arc, RwLock};

use crate::cache::engine::EngineClusterCache;

pub struct PreferredElection {
    engine_cache: Arc<RwLock<EngineClusterCache>>,
}

impl PreferredElection {
    pub fn new(engine_cache: Arc<RwLock<EngineClusterCache>>) -> Self {
        return PreferredElection { engine_cache };
    }

    pub async fn start(&self) {
        
    }
}
