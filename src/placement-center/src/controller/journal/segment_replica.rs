use crate::{
    cache::{placement::PlacementCacheManager, journal::JournalCacheManager}, storage::journal::segment::Replica,
};
use std::sync::Arc;

pub struct SegmentReplicaAlgorithm {
    cluster_cache: Arc<PlacementCacheManager>,
    engine_cache: Arc<JournalCacheManager>,
}

impl SegmentReplicaAlgorithm {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        engine_cache: Arc<JournalCacheManager>,
    ) -> SegmentReplicaAlgorithm {
        return SegmentReplicaAlgorithm {
            cluster_cache,
            engine_cache,
        };
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
