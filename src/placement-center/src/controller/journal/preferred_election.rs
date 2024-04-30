use std::sync::{Arc, RwLock};

use crate::cache::journal::JournalCache;

pub struct PreferredElection {
    engine_cache: Arc<RwLock<JournalCache>>,
}

impl PreferredElection {
    pub fn new(engine_cache: Arc<RwLock<JournalCache>>) -> Self {
        return PreferredElection { engine_cache };
    }

    pub async fn start(&self) {}
}
