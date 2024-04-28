use std::sync::{Arc, RwLock};

use crate::cache::engine::EngineCache;

pub struct PreferredElection {
    engine_cache: Arc<RwLock<EngineCache>>,
}

impl PreferredElection {
    pub fn new(engine_cache: Arc<RwLock<EngineCache>>) -> Self {
        return PreferredElection { engine_cache };
    }

    pub async fn start(&self) {}
}
