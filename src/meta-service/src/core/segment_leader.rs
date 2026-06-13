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
        cache::MetaCacheManager, error::MetaServiceError, notify::send_notify_by_set_segment,
        segment::sync_save_segment_info,
    },
    raft::manager::MultiRaftManager,
    storage::common::node::NodeStorage,
};
use metadata_struct::storage::segment::{EngineSegment, SegmentStatus};
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use tracing::{info, warn};

// Number of segments handled by a single switch task. Affected segments are
// split into chunks of this size and processed concurrently; a batch completes
// once all its tasks complete.
const SWITCH_TASK_CHUNK_SIZE: usize = 1000;

pub async fn segment_leader_switch(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    remove_id: u64,
) -> Result<(), MetaServiceError> {
    let affected: Vec<EngineSegment> = meta_cache
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

    // Active (write-accepting) segments are on the hot write path; sealed /
    // historical segments only serve reads. Switch them on two independent
    // tasks so the active switch never waits behind the (potentially huge)
    // set of inactive segments.
    let (active, inactive): (Vec<_>, Vec<_>) =
        affected.into_iter().partition(|seg| seg.allow_write());

    let active_task = tokio::spawn(active_segment_leader_switch(
        raft_manager.clone(),
        call_manager.clone(),
        rocksdb_engine_handler.clone(),
        remove_id,
        active,
    ));
    let inactive_task = tokio::spawn(inactive_segment_leader_switch(
        raft_manager.clone(),
        call_manager.clone(),
        rocksdb_engine_handler.clone(),
        remove_id,
        inactive,
    ));

    let (active_res, inactive_res) = tokio::join!(active_task, inactive_task);
    join_result(active_res)?;
    join_result(inactive_res)?;
    Ok(())
}

async fn active_segment_leader_switch(
    raft_manager: Arc<MultiRaftManager>,
    call_manager: Arc<NodeCallManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    remove_id: u64,
    segments: Vec<EngineSegment>,
) -> Result<(), MetaServiceError> {
    switch_segments_concurrently(
        "active",
        raft_manager,
        call_manager,
        rocksdb_engine_handler,
        remove_id,
        segments,
    )
    .await
}

async fn inactive_segment_leader_switch(
    raft_manager: Arc<MultiRaftManager>,
    call_manager: Arc<NodeCallManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    remove_id: u64,
    segments: Vec<EngineSegment>,
) -> Result<(), MetaServiceError> {
    switch_segments_concurrently(
        "inactive",
        raft_manager,
        call_manager,
        rocksdb_engine_handler,
        remove_id,
        segments,
    )
    .await
}

async fn switch_segments_concurrently(
    label: &str,
    raft_manager: Arc<MultiRaftManager>,
    call_manager: Arc<NodeCallManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    remove_id: u64,
    segments: Vec<EngineSegment>,
) -> Result<(), MetaServiceError> {
    let mut handles = Vec::new();
    for chunk in segments.chunks(SWITCH_TASK_CHUNK_SIZE) {
        let chunk = chunk.to_vec();
        let raft_manager = raft_manager.clone();
        let call_manager = call_manager.clone();
        let rocksdb_engine_handler = rocksdb_engine_handler.clone();
        handles.push(tokio::spawn(async move {
            switch_segment_chunk(
                &raft_manager,
                &call_manager,
                &rocksdb_engine_handler,
                remove_id,
                chunk,
            )
            .await
        }));
    }

    // The batch completes only once every task completes; await them all
    // before propagating the first error.
    let mut switched = 0u32;
    let mut unavailable = 0u32;
    let mut first_err = None;
    for handle in handles {
        match handle.await {
            Ok(Ok((s, u))) => {
                switched += s;
                unavailable += u;
            }
            Ok(Err(e)) => first_err = first_err.or(Some(e)),
            Err(e) => {
                first_err = first_err.or(Some(MetaServiceError::CommonError(format!(
                    "{label} segment leader switch task panicked: {e}"
                ))))
            }
        }
    }
    if let Some(e) = first_err {
        return Err(e);
    }
    info!(
        "{label}_segment_leader_switch completed, node {} removed, {} switched, {} marked Unavailable",
        remove_id, switched, unavailable
    );
    Ok(())
}

async fn switch_segment_chunk(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    remove_id: u64,
    segments: Vec<EngineSegment>,
) -> Result<(u32, u32), MetaServiceError> {
    let node_storage = NodeStorage::new(rocksdb_engine_handler.clone());

    let mut switched = 0u32;
    let mut unavailable = 0u32;
    for segment in segments {
        let new_leader = segment.isr.iter().copied().find(|id| *id != remove_id);
        let new_leader_broker_epoch = match new_leader {
            Some(id) => node_storage.get_broker_epoch(id)?,
            None => 0,
        };
        let new_segment =
            compute_segment_after_leader_failure(&segment, remove_id, new_leader_broker_epoch);

        if new_segment.status == SegmentStatus::Unavailable {
            warn!(
                "segment {}/{} marked Unavailable after removing leader {}: no surviving ISR member (ISR was {:?}, segment_epoch {} -> {})",
                segment.shard_name,
                segment.segment_seq,
                remove_id,
                segment.isr,
                segment.segment_epoch,
                new_segment.segment_epoch
            );
            unavailable += 1;
        } else {
            info!(
                "segment {}/{} leader switched {} -> {} after removing node {} (leader_epoch {} -> {}, segment_epoch {} -> {}, ISR {:?} -> {:?})",
                segment.shard_name,
                segment.segment_seq,
                segment.leader,
                new_segment.leader,
                remove_id,
                segment.leader_epoch,
                new_segment.leader_epoch,
                segment.segment_epoch,
                new_segment.segment_epoch,
                segment.isr,
                new_segment.isr
            );
            switched += 1;
        }

        sync_save_segment_info(raft_manager, &new_segment).await?;
        send_notify_by_set_segment(call_manager, new_segment).await?;
    }
    Ok((switched, unavailable))
}

fn join_result(
    res: Result<Result<(), MetaServiceError>, tokio::task::JoinError>,
) -> Result<(), MetaServiceError> {
    match res {
        Ok(inner) => inner,
        Err(e) => Err(MetaServiceError::CommonError(format!(
            "segment leader switch task panicked: {e}"
        ))),
    }
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
