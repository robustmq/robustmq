// Copyright 2023 RobustMQ Team
//
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

use crate::core::cache::MetaCacheManager;
use crate::core::error::MetaServiceError;
use crate::storage::common::node::NodeStorage;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use crate::storage::journal::shard::ShardStorage;
use bytes::Bytes;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use prost::Message as _;
use protocol::meta::meta_service_journal::UpdateSegmentIsrRequest;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Outcome of an ISR update applied by the state machine, encoded into the raft
/// response value as JSON. A rejection is a normal result (not a fault), so it
/// travels through the `Ok` channel rather than an apply `Err`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsrUpdateOutcome {
    Applied(u32),
    SegmentNotFound,
    NotLeader,
    FencedLeaderEpoch,
    StaleBrokerEpoch,
    InvalidUpdateVersion,
    InvalidIsr,
}

impl IsrUpdateOutcome {
    pub fn encode(&self) -> Bytes {
        Bytes::from(serde_json::to_vec(self).expect("serialize IsrUpdateOutcome"))
    }

    pub fn decode(value: &[u8]) -> Result<Self, MetaServiceError> {
        serde_json::from_slice(value).map_err(|e| {
            MetaServiceError::CommonError(format!("malformed IsrUpdateOutcome response: {e}"))
        })
    }
}

#[derive(Clone)]
pub struct DataRouteJournal {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<MetaCacheManager>,
}

impl DataRouteJournal {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<MetaCacheManager>,
    ) -> Self {
        DataRouteJournal {
            rocksdb_engine_handler,
            cache_manager,
        }
    }

    pub async fn set_shard(&self, value: Bytes) -> Result<Bytes, MetaServiceError> {
        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        let shard_info = EngineShard::decode(&value)?;
        shard_storage.save(&shard_info)?;
        self.cache_manager.set_shard(shard_info);
        Ok(value)
    }

    pub async fn delete_shard(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let shard_info = EngineShard::decode(&value)?;

        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        shard_storage.delete(&shard_info.shard_name)?;

        self.cache_manager.remove_shard(&shard_info.shard_name);

        Ok(())
    }

    pub async fn set_segment(&self, value: Bytes) -> Result<Bytes, MetaServiceError> {
        let segment = EngineSegment::decode(&value)?;

        let storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(segment.clone())?;
        self.cache_manager.set_segment(segment);

        Ok(value)
    }

    pub async fn update_segment_isr(&self, value: Bytes) -> Result<Bytes, MetaServiceError> {
        let req = UpdateSegmentIsrRequest::decode(value.as_ref())?;
        let storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());

        let Some(mut current) = storage.get(&req.shard_name, req.segment)? else {
            return Ok(IsrUpdateOutcome::SegmentNotFound.encode());
        };

        if req.requester_node_id != current.leader {
            return Ok(IsrUpdateOutcome::NotLeader.encode());
        }

        if req.expected_leader_epoch != current.leader_epoch {
            return Ok(IsrUpdateOutcome::FencedLeaderEpoch.encode());
        }

        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        let known_broker_epoch = node_storage.get_broker_epoch(req.requester_node_id)?;
        if req.requester_broker_epoch != known_broker_epoch {
            return Ok(IsrUpdateOutcome::StaleBrokerEpoch.encode());
        }

        if req.expected_segment_epoch != current.segment_epoch {
            return Ok(IsrUpdateOutcome::InvalidUpdateVersion.encode());
        }

        let replica_ids: Vec<u64> = current.replicas.iter().map(|r| r.node_id).collect();
        if req.new_isr.is_empty()
            || !req.new_isr.contains(&current.leader)
            || !req.new_isr.iter().all(|n| replica_ids.contains(n))
        {
            return Ok(IsrUpdateOutcome::InvalidIsr.encode());
        }

        current.isr = req.new_isr;
        current.segment_epoch += 1;
        let new_segment_epoch = current.segment_epoch;

        storage.save(current.clone())?;
        self.cache_manager.set_segment(current);

        Ok(IsrUpdateOutcome::Applied(new_segment_epoch).encode())
    }

    pub async fn delete_segment(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let segment = EngineSegment::decode(&value)?;

        let storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&segment.shard_name, segment.segment_seq)?;

        self.cache_manager
            .remove_segment(&segment.shard_name, segment.segment_seq);
        Ok(())
    }

    pub async fn set_segment_meta(&self, value: Bytes) -> Result<Bytes, MetaServiceError> {
        let meta = EngineSegmentMetadata::decode(&value)?;

        let storage = SegmentMetadataStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(meta.clone())?;
        self.cache_manager.set_segment_meta(meta);
        Ok(value)
    }

    pub async fn delete_segment_meta(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let meta = EngineSegmentMetadata::decode(&value)?;

        let storage = SegmentMetadataStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&meta.shard_name, meta.segment_seq)?;

        self.cache_manager
            .remove_segment_meta(&meta.shard_name, meta.segment_seq);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::storage::segment::{Replica, SegmentStatus};
    use rocksdb_engine::storage::family::column_family_list;

    fn setup() -> DataRouteJournal {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            100_000,
            column_family_list(),
        ));
        let cache = Arc::new(MetaCacheManager::new(rocksdb_engine.clone()));
        DataRouteJournal::new(rocksdb_engine, cache)
    }

    fn seed_segment(journal: &DataRouteJournal, leader: u64, replicas: Vec<u64>) -> EngineSegment {
        let segment = EngineSegment {
            shard_name: "s1".to_string(),
            segment_seq: 0,
            leader,
            leader_epoch: 3,
            segment_epoch: 0,
            isr: replicas.clone(),
            replicas: replicas
                .into_iter()
                .map(|node_id| Replica {
                    replica_seq: 0,
                    node_id,
                    fold: String::new(),
                })
                .collect(),
            status: SegmentStatus::Write,
            ..Default::default()
        };
        let storage = SegmentStorage::new(journal.rocksdb_engine_handler.clone());
        storage.save(segment.clone()).unwrap();
        segment
    }

    fn req(seg: &EngineSegment, new_isr: Vec<u64>) -> UpdateSegmentIsrRequest {
        UpdateSegmentIsrRequest {
            shard_name: seg.shard_name.clone(),
            segment: seg.segment_seq,
            new_isr,
            requester_node_id: seg.leader,
            requester_broker_epoch: 0,
            expected_leader_epoch: seg.leader_epoch,
            expected_segment_epoch: seg.segment_epoch,
        }
    }

    async fn apply(journal: &DataRouteJournal, r: &UpdateSegmentIsrRequest) -> IsrUpdateOutcome {
        let out = journal
            .update_segment_isr(Bytes::from(r.encode_to_vec()))
            .await
            .unwrap();
        IsrUpdateOutcome::decode(&out).unwrap()
    }

    #[tokio::test]
    async fn success_shrinks_isr_and_bumps_epoch() {
        let j = setup();
        let node_storage = NodeStorage::new(j.rocksdb_engine_handler.clone());
        node_storage.next_broker_epoch(1).unwrap();
        let seg = seed_segment(&j, 1, vec![1, 2, 3]);

        let mut r = req(&seg, vec![1, 2]);
        r.requester_broker_epoch = 1;
        assert!(matches!(apply(&j, &r).await, IsrUpdateOutcome::Applied(1)));

        let stored = SegmentStorage::new(j.rocksdb_engine_handler.clone())
            .get("s1", 0)
            .unwrap()
            .unwrap();
        assert_eq!(stored.isr, vec![1, 2]);
        assert_eq!(stored.segment_epoch, 1);
    }

    #[tokio::test]
    async fn fences_reject_without_returning_err() {
        let j = setup();
        let seg = seed_segment(&j, 1, vec![1, 2, 3]);

        let with = |f: &dyn Fn(&mut UpdateSegmentIsrRequest)| {
            let mut r = req(&seg, vec![1, 2]);
            f(&mut r);
            r
        };

        assert!(matches!(
            apply(&j, &with(&|r| r.requester_node_id = 2)).await,
            IsrUpdateOutcome::NotLeader
        ));
        assert!(matches!(
            apply(&j, &with(&|r| r.expected_leader_epoch = 99)).await,
            IsrUpdateOutcome::FencedLeaderEpoch
        ));
        assert!(matches!(
            apply(&j, &with(&|r| r.requester_broker_epoch = 7)).await,
            IsrUpdateOutcome::StaleBrokerEpoch
        ));
        assert!(matches!(
            apply(&j, &with(&|r| r.expected_segment_epoch = 42)).await,
            IsrUpdateOutcome::InvalidUpdateVersion
        ));
        // ISR validity re-checked in apply, not just the server layer (C2)
        assert!(matches!(
            apply(&j, &req(&seg, vec![])).await,
            IsrUpdateOutcome::InvalidIsr
        ));
        assert!(matches!(
            apply(&j, &req(&seg, vec![2, 3])).await,
            IsrUpdateOutcome::InvalidIsr
        ));
        assert!(matches!(
            apply(&j, &req(&seg, vec![1, 9])).await,
            IsrUpdateOutcome::InvalidIsr
        ));
    }

    #[tokio::test]
    async fn concurrent_update_only_matching_cas_wins() {
        let j = setup();
        let seg = seed_segment(&j, 1, vec![1, 2, 3]);
        assert!(matches!(
            apply(&j, &req(&seg, vec![1, 2])).await,
            IsrUpdateOutcome::Applied(1)
        ));
        assert!(matches!(
            apply(&j, &req(&seg, vec![1, 3])).await,
            IsrUpdateOutcome::InvalidUpdateVersion
        ));
    }
}
