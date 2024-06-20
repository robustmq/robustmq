use std::sync::{Arc, RwLock};

use crate::cache::journal::JournalCacheManager;

pub struct PreferredElection {
    engine_cache: Arc<RwLock<JournalCacheManager>>,
}

impl PreferredElection {
    pub fn new(engine_cache: Arc<RwLock<JournalCacheManager>>) -> Self {
        return PreferredElection { engine_cache };
    }

    pub async fn start(&self) {}
}
