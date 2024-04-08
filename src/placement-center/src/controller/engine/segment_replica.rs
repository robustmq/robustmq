use std::sync::{Arc, RwLock};
use crate::{
    cache::{cluster::ClusterCache, engine::EngineCache},
    storage::segment::Replica,
};

pub struct SegmentReplicaAlgorithm {
    cluster_cache: Arc<RwLock<ClusterCache>>,
    engine_cache: Arc<RwLock<EngineCache>>,
}

impl SegmentReplicaAlgorithm {
    pub fn new(
        cluster_cache: Arc<RwLock<ClusterCache>>,
        engine_cache: Arc<RwLock<EngineCache>>,
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
