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
use crate::core::notify::send_notify_by_set_segment;
use crate::core::segment::{calc_node_fold, sync_save_segment_info};
use crate::raft::manager::MultiRaftManager;
use crate::storage::common::node::NodeStorage;
use metadata_struct::storage::segment::{EngineSegment, Replica, SegmentStatus};
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

// Number of segments handled by a single decommission task. Affected segments
// are split into chunks of this size and processed concurrently; the pass
// completes once all tasks complete.
const DECOMMISSION_TASK_CHUNK_SIZE: usize = 1000;

/// Permanently decommission `remove_id`: for every segment the node is a replica
/// of, drop it from the ISR and replica set, re-elect a leader if it was the
/// leader, and migrate the vacated replica slot to a surviving node (metadata
/// only — the new replica catches up from the leader on its own).
///
/// The node must already be removed from `node_list` before this runs, so it is
/// never picked as a migration target.
pub async fn decommission_node_segments(
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
                .filter(|seg| {
                    seg.leader == remove_id || seg.replicas.iter().any(|r| r.node_id == remove_id)
                })
                .map(|seg| seg.clone())
                .collect::<Vec<_>>()
        })
        .collect();
    if affected.is_empty() {
        return Ok(());
    }

    let alive_ids: Arc<Vec<u64>> = Arc::new(
        meta_cache
            .get_engine_node_list()
            .iter()
            .map(|n| n.node_id)
            .collect(),
    );
    let load = Arc::new(Mutex::new(replica_load(meta_cache, &alive_ids)));

    let mut handles = Vec::new();
    for chunk in affected.chunks(DECOMMISSION_TASK_CHUNK_SIZE) {
        let chunk = chunk.to_vec();
        let meta_cache = meta_cache.clone();
        let raft_manager = raft_manager.clone();
        let call_manager = call_manager.clone();
        let rocksdb_engine_handler = rocksdb_engine_handler.clone();
        let alive_ids = alive_ids.clone();
        let load = load.clone();
        handles.push(tokio::spawn(async move {
            decommission_chunk(
                &meta_cache,
                &raft_manager,
                &call_manager,
                &rocksdb_engine_handler,
                remove_id,
                chunk,
                &alive_ids,
                &load,
            )
            .await
        }));
    }

    let mut migrated = 0u32;
    let mut unavailable = 0u32;
    let mut first_err = None;
    for handle in handles {
        match handle.await {
            Ok(Ok((m, u))) => {
                migrated += m;
                unavailable += u;
            }
            Ok(Err(e)) => first_err = first_err.or(Some(e)),
            Err(e) => {
                first_err = first_err.or(Some(MetaServiceError::CommonError(format!(
                    "decommission segment task panicked: {e}"
                ))))
            }
        }
    }
    if let Some(e) = first_err {
        return Err(e);
    }
    info!(
        "node {} decommission completed: {} segments migrated, {} marked Unavailable",
        remove_id, migrated, unavailable
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn decommission_chunk(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    remove_id: u64,
    segments: Vec<EngineSegment>,
    alive_ids: &[u64],
    load: &Mutex<HashMap<u64, u64>>,
) -> Result<(u32, u32), MetaServiceError> {
    let node_storage = NodeStorage::new(rocksdb_engine_handler.clone());

    let mut migrated = 0u32;
    let mut unavailable = 0u32;
    for segment in segments {
        let existing: HashSet<u64> = segment.replicas.iter().map(|r| r.node_id).collect();
        let target = pick_target(alive_ids, load, &existing);

        let new_segment =
            build_decommissioned_segment(meta_cache, &node_storage, &segment, remove_id, target)?;

        if new_segment.status == SegmentStatus::Unavailable {
            warn!(
                "segment {}/{} marked Unavailable after decommissioning leader {}: no surviving ISR member",
                segment.shard_name, segment.segment_seq, remove_id
            );
            unavailable += 1;
        } else {
            migrated += 1;
        }

        sync_save_segment_info(raft_manager, &new_segment).await?;
        send_notify_by_set_segment(call_manager, new_segment).await?;
    }
    Ok((migrated, unavailable))
}

/// Pick the least-loaded surviving node that is not already a replica of the
/// segment, and charge it one replica. Returns None when no node can take it
/// (every alive node already hosts a replica, or the cluster is empty).
fn pick_target(
    alive_ids: &[u64],
    load: &Mutex<HashMap<u64, u64>>,
    existing: &HashSet<u64>,
) -> Option<u64> {
    let mut guard = load.lock().unwrap();
    let target = alive_ids
        .iter()
        .filter(|id| !existing.contains(id))
        .min_by_key(|id| (*guard.get(id).unwrap_or(&0), **id))
        .copied()?;
    *guard.entry(target).or_insert(0) += 1;
    Some(target)
}

fn build_decommissioned_segment(
    meta_cache: &Arc<MetaCacheManager>,
    node_storage: &NodeStorage,
    segment: &EngineSegment,
    remove_id: u64,
    target: Option<u64>,
) -> Result<EngineSegment, MetaServiceError> {
    let mut new_segment = segment.clone();
    new_segment.isr.retain(|id| *id != remove_id);
    new_segment.replicas.retain(|r| r.node_id != remove_id);

    // Metadata-only migration: the new replica joins the replica set but not the
    // ISR; it becomes in-sync only after catching up from the leader.
    if let Some(target) = target {
        let fold = calc_node_fold(meta_cache, target)?;
        let next_seq = new_segment
            .replicas
            .iter()
            .map(|r| r.replica_seq)
            .max()
            .map_or(0, |m| m + 1);
        new_segment.replicas.push(Replica {
            replica_seq: next_seq,
            node_id: target,
            fold,
        });
    }

    new_segment.segment_epoch += 1;

    if segment.leader == remove_id {
        match new_segment.isr.first().copied() {
            Some(new_leader) => {
                new_segment.leader = new_leader;
                new_segment.leader_epoch += 1;
                new_segment.leader_broker_epoch = node_storage.get_broker_epoch(new_leader)?;
            }
            None => {
                if !segment.isr.is_empty() {
                    new_segment.last_known_isr = segment.isr.clone();
                }
                new_segment.status = SegmentStatus::Unavailable;
            }
        }
    }

    Ok(new_segment)
}

fn replica_load(meta_cache: &Arc<MetaCacheManager>, alive_ids: &[u64]) -> HashMap<u64, u64> {
    let mut counts: HashMap<u64, u64> = alive_ids.iter().map(|id| (*id, 0)).collect();
    for shard in meta_cache.segment_list.iter() {
        for seg in shard.iter() {
            for replica in &seg.replicas {
                if let Some(count) = counts.get_mut(&replica.node_id) {
                    *count += 1;
                }
            }
        }
    }
    counts
}
