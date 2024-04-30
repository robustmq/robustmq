use crate::{
    cache::{cluster::ClusterCache, journal::JournalCache},
    storage::segment::Replica,
};
use std::sync::Arc;

pub struct SegmentReplicaAlgorithm {
    cluster_cache: Arc<ClusterCache>,
    engine_cache: Arc<JournalCache>,
}

impl SegmentReplicaAlgorithm {
    pub fn new(
        cluster_cache: Arc<ClusterCache>,
        engine_cache: Arc<JournalCache>,
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
