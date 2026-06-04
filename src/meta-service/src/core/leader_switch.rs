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

use std::sync::Arc;

use crate::{
    core::{
        cache::MetaCacheManager,
        error::MetaServiceError,
        group_leader::generate_group_leader,
        notify::{send_notify_by_set_segment, send_notify_by_set_share_group},
        segment::sync_save_segment_info,
    },
    raft::{
        manager::MultiRaftManager,
        route::data::{StorageData, StorageDataType},
    },
    storage::common::node::NodeStorage,
};
use bytes::Bytes;
use metadata_struct::storage::segment::{EngineSegment, SegmentStatus};
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use tracing::{error, info, warn};

pub async fn trigger_leader_switch(
    meta_cache: Arc<MetaCacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_call_manager: Arc<NodeCallManager>,
    remove_id: u64,
) {
    tokio::spawn(async move {
        let result: Result<(), MetaServiceError> = async {
            group_leader_switch(
                &meta_cache,
                &raft_manager,
                &mqtt_call_manager,
                &rocksdb_engine_handler,
                remove_id,
            )
            .await?;
            segment_leader_switch(
                &meta_cache,
                &raft_manager,
                &mqtt_call_manager,
                &rocksdb_engine_handler,
                remove_id,
            )
            .await?;
            Ok(())
        }
        .await;
        if let Err(e) = result {
            error!("leader switch failed for removed node {}: {}", remove_id, e);
        }
    });
}

pub async fn group_leader_switch(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    remove_id: u64,
) -> Result<(), MetaServiceError> {
    let affected: Vec<_> = meta_cache
        .group_leader
        .iter()
        .filter(|g| g.leader_broker == remove_id)
        .map(|g| g.clone())
        .collect();

    let mut switched = 0u32;
    for mut group_leader in affected {
        let new_leader_broker =
            generate_group_leader(meta_cache, rocksdb_engine_handler, &group_leader.tenant).await?;
        group_leader.leader_broker = new_leader_broker;

        let data = StorageData::new(
            StorageDataType::MqttSetGroupLeader,
            Bytes::copy_from_slice(&group_leader.encode()?),
        );
        raft_manager
            .write_data(&group_leader.group_name, data)
            .await?;
        send_notify_by_set_share_group(call_manager, group_leader).await?;
        switched += 1;
    }
    info!(
        "group_leader_switch completed, node {} removed, {} group leaders switched",
        remove_id, switched
    );
    Ok(())
}

pub async fn segment_leader_switch(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    remove_id: u64,
) -> Result<(), MetaServiceError> {
    let affected: Vec<_> = meta_cache
        .segment_list
        .iter()
        .flat_map(|shard| {
            shard
                .iter()
                .filter(|seg| seg.leader == remove_id)
                .map(|seg| seg.clone())
                .collect::<Vec<_>>()
        })
        .collect();

    let node_storage = NodeStorage::new(rocksdb_engine_handler.clone());

    let mut switched = 0u32;
    let mut unavailable = 0u32;
    for segment in affected {
        let new_leader = segment.isr.iter().copied().find(|id| *id != remove_id);
        let new_leader_broker_epoch = match new_leader {
            Some(id) => node_storage.get_broker_epoch(id)?,
            None => 0,
        };
        let new_segment: EngineSegment =
            compute_segment_after_leader_failure(&segment, remove_id, new_leader_broker_epoch);

        if new_segment.status == SegmentStatus::Unavailable {
            warn!(
                "segment {}/{} ISR empty after removing node {}, marking Unavailable",
                segment.shard_name, segment.segment_seq, remove_id
            );
            unavailable += 1;
        } else {
            switched += 1;
        }

        sync_save_segment_info(raft_manager, &new_segment).await?;
        send_notify_by_set_segment(call_manager, new_segment).await?;
    }
    info!(
        "segment_leader_switch completed, node {} removed, {} switched, {} marked Unavailable",
        remove_id, switched, unavailable
    );
    Ok(())
}

/// Decide a segment's new state after its leader `remove_id` fails. Elects the
/// first surviving ISR member (never a replica outside ISR, which could be
/// missing committed data). With no surviving ISR member the segment becomes
/// Unavailable rather than risking an unclean election.
fn compute_segment_after_leader_failure(
    segment: &EngineSegment,
    remove_id: u64,
    new_leader_broker_epoch: u64,
) -> EngineSegment {
    let mut new_segment = segment.clone();
    let new_leader = segment.isr.iter().copied().find(|id| *id != remove_id);

    new_segment.isr.retain(|id| *id != remove_id);
    new_segment.segment_epoch += 1;

    match new_leader {
        Some(leader) => {
            new_segment.leader = leader;
            new_segment.leader_epoch += 1;
            new_segment.leader_broker_epoch = new_leader_broker_epoch;
        }
        None => {
            if !segment.isr.is_empty() {
                new_segment.last_known_isr = segment.isr.clone();
            }
            new_segment.status = SegmentStatus::Unavailable;
        }
    }
    new_segment
}

#[cfg(test)]
mod tests {
    use super::compute_segment_after_leader_failure;
    use metadata_struct::storage::segment::{EngineSegment, Replica, SegmentStatus};

    fn segment(leader: u64, isr: Vec<u64>, replicas: Vec<u64>) -> EngineSegment {
        EngineSegment {
            shard_name: "s1".to_string(),
            segment_seq: 0,
            leader,
            leader_epoch: 5,
            segment_epoch: 9,
            isr,
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
        }
    }

    #[test]
    fn elects_from_isr_and_bumps_epochs() {
        let seg = segment(1, vec![1, 2, 3], vec![1, 2, 3]);
        let out = compute_segment_after_leader_failure(&seg, 1, 7);

        assert_eq!(out.leader, 2);
        assert_eq!(out.leader_epoch, 6);
        assert_eq!(out.segment_epoch, 10);
        assert_eq!(out.leader_broker_epoch, 7);
        assert_eq!(out.isr, vec![2, 3]);
        assert_eq!(out.status, SegmentStatus::Write);
    }

    #[test]
    fn never_elects_replica_outside_isr() {
        let seg = segment(1, vec![1], vec![1, 2, 3]);
        let out = compute_segment_after_leader_failure(&seg, 1, 0);

        assert_eq!(out.status, SegmentStatus::Unavailable);
        assert!(out.isr.is_empty());
        assert_eq!(out.last_known_isr, vec![1]);
        assert_eq!(out.segment_epoch, 10);
        assert_eq!(out.leader_epoch, 5);
        assert_eq!(out.leader, 1);
    }

    #[test]
    fn empty_isr_marks_unavailable_with_last_known() {
        let seg = segment(1, vec![1, 2], vec![1, 2, 3]);
        let out = compute_segment_after_leader_failure(&seg, 1, 0);
        assert_eq!(out.leader, 2);

        let seg2 = segment(2, vec![2], vec![1, 2, 3]);
        let out2 = compute_segment_after_leader_failure(&seg2, 2, 0);
        assert_eq!(out2.status, SegmentStatus::Unavailable);
        assert_eq!(out2.last_known_isr, vec![2]);
    }
}
