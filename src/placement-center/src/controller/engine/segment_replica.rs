use std::sync::{Arc, RwLock};

use crate::cache::engine::EngineClusterCache;

pub struct SegmentReplicaAlgorithm {
    engine_cache: Arc<RwLock<EngineClusterCache>>,
}

impl SegmentReplicaAlgorithm {
    pub fn new(engine_cache: Arc<RwLock<EngineClusterCache>>) -> SegmentReplicaAlgorithm {
        return SegmentReplicaAlgorithm { engine_cache };
    }

    pub fn calc_replica_distribution(&self) -> Vec<u64> {
        return vec![1];
    }
}
