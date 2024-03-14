use std::sync::{Arc, RwLock};

use crate::{cache::engine::EngineClusterCache, storage::segment::Replica};

pub struct SegmentReplicaAlgorithm {
    engine_cache: Arc<RwLock<EngineClusterCache>>,
}

impl SegmentReplicaAlgorithm {
    pub fn new(engine_cache: Arc<RwLock<EngineClusterCache>>) -> SegmentReplicaAlgorithm {
        return SegmentReplicaAlgorithm { engine_cache };
    }

    pub fn calc_replica_distribution(&self, replica_seq: u64) -> Vec<Replica> {
        let node_id = 1;
        let fold = "/data/robustmq".to_string();
        let rep = Replica {
            replica_seq,
            node_id,
            fold,
        };
        return vec![rep];
    }
}
