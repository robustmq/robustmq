// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use crate::{
    cache::{placement::PlacementCacheManager, journal::JournalCacheManager}, storage::journal::segment::Replica,
};
use std::sync::Arc;

pub struct SegmentReplicaAlgorithm {
    #[allow(dead_code)]
    cluster_cache: Arc<PlacementCacheManager>,
    #[allow(dead_code)]
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
