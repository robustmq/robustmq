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
use crate::core::notify::send_notify_by_set_segment;
use crate::core::segment::sync_save_segment_info;
use crate::raft::manager::MultiRaftManager;
use dashmap::DashMap;
use grpc_clients::broker::common::call::broker_query_replica_leo;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::storage::segment::{EngineSegment, SegmentStatus};
use node_call::NodeCallManager;
use protocol::broker::broker::QueryReplicaLeoRequest;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex as AsyncMutex;
use tracing::{info, warn};

static RECOVERY_LOCKS: OnceLock<DashMap<String, Arc<AsyncMutex<()>>>> = OnceLock::new();

fn recovery_lock(segment_key: &str) -> Arc<AsyncMutex<()>> {
    RECOVERY_LOCKS
        .get_or_init(DashMap::new)
        .entry(segment_key.to_string())
        .or_insert_with(|| Arc::new(AsyncMutex::new(())))
        .clone()
}

pub async fn recover_unavailable_segments_on_node_join(
    node_id: u64,
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    client_pool: &Arc<ClientPool>,
) {
    // Only the active segment needs a leader; sealed segments are read-only.
    let to_recover: Vec<EngineSegment> = meta_cache
        .shard_list
        .iter()
        .filter_map(|shard| meta_cache.get_segment(&shard.shard_name, shard.active_segment_seq))
        .filter(|seg| {
            meta_cache.get_broker_node(seg.leader).is_none()
                || (seg.status == SegmentStatus::Unavailable
                    && seg.last_known_isr.contains(&node_id))
        })
        .collect();

    for segment in to_recover {
        if let Err(e) = try_recover_unavailable_segment(
            &segment,
            meta_cache,
            raft_manager,
            call_manager,
            client_pool,
        )
        .await
        {
            warn!(
                "isr recovery failed for {}/{}: {}",
                segment.shard_name, segment.segment_seq, e
            );
        }
    }
}

async fn try_recover_unavailable_segment(
    segment: &EngineSegment,
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), String> {
    let key = format!("{}/{}", segment.shard_name, segment.segment_seq);
    let lock = recovery_lock(&key);
    let _guard = lock.lock().await;

    // Needs recovery: parked Unavailable, or its leader is no longer a live node.
    let needs_recovery = |seg: &EngineSegment| {
        seg.status == SegmentStatus::Unavailable || meta_cache.get_broker_node(seg.leader).is_none()
    };

    let current = match meta_cache.get_segment(&segment.shard_name, segment.segment_seq) {
        Some(s) => s,
        None => return Ok(()),
    };

    if !needs_recovery(&current) {
        return Ok(());
    }

    let candidate_ids: &[u64] = if current.status == SegmentStatus::Unavailable {
        &current.last_known_isr
    } else {
        &current.isr
    };

    let nodes: Vec<BrokerNode> = meta_cache
        .node_list
        .iter()
        .filter(|n| candidate_ids.contains(n.key()))
        .map(|n| n.value().clone())
        .collect();
    if nodes.is_empty() {
        return Ok(());
    }

    let mut leo_map: HashMap<u64, ReplicaStateReport> = HashMap::new();
    for node in &nodes {
        let req = QueryReplicaLeoRequest {
            shard_name: current.shard_name.clone(),
            segment_seq: current.segment_seq,
        };
        match broker_query_replica_leo(client_pool, &[&node.grpc_addr], req).await {
            Ok(reply) => {
                leo_map.insert(
                    node.node_id,
                    ReplicaStateReport {
                        replica_id: node.node_id,
                        segment_leo: reply.segment_leo,
                        latest_leader_epoch: reply.latest_leader_epoch,
                        available: reply.available,
                    },
                );
            }
            Err(e) => {
                warn!("query_replica_leo node {}: {}", node.node_id, e);
            }
        }
    }

    let reports: Vec<ReplicaStateReport> = leo_map.into_values().collect();
    let new_leader_id = match elect_recovery_leader(&reports) {
        Some(id) => id,
        None => return Ok(()),
    };

    // Leader unchanged on an already-writable segment: nothing to update.
    if new_leader_id == current.leader && current.status != SegmentStatus::Unavailable {
        return Ok(());
    }

    let mut new_segment = current.clone();
    new_segment.leader = new_leader_id;
    new_segment.leader_epoch += 1;
    new_segment.segment_epoch += 1;
    new_segment.isr = vec![new_leader_id];
    new_segment.status = SegmentStatus::Write;
    new_segment.last_known_isr.clear();

    sync_save_segment_info(raft_manager, &new_segment)
        .await
        .map_err(|e| e.to_string())?;

    // Cross-node CAS check: verify our write won (another meta node may have raced us).
    if let Some(after) = meta_cache.get_segment(&current.shard_name, current.segment_seq) {
        if after.leader != new_leader_id {
            warn!(
                "isr recovery: {}/{} write race — another node elected leader={} instead of {}",
                current.shard_name, current.segment_seq, after.leader, new_leader_id
            );
            return Ok(());
        }
    }

    send_notify_by_set_segment(call_manager, new_segment.clone())
        .await
        .map_err(|e| e.to_string())?;

    info!(
        "isr recovery: {}/{} recovered, leader {} -> {}",
        current.shard_name, current.segment_seq, current.leader, new_leader_id
    );
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplicaStateReport {
    pub replica_id: u64,
    pub segment_leo: u64,
    pub latest_leader_epoch: u32,
    pub available: bool,
}

pub fn elect_recovery_leader(reports: &[ReplicaStateReport]) -> Option<u64> {
    reports
        .iter()
        .filter(|r| r.available)
        .max_by(|a, b| {
            a.segment_leo
                .cmp(&b.segment_leo)
                .then(a.latest_leader_epoch.cmp(&b.latest_leader_epoch))
                .then(b.replica_id.cmp(&a.replica_id))
        })
        .map(|r| r.replica_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(
        replica_id: u64,
        segment_leo: u64,
        epoch: u32,
        available: bool,
    ) -> ReplicaStateReport {
        ReplicaStateReport {
            replica_id,
            segment_leo,
            latest_leader_epoch: epoch,
            available,
        }
    }

    #[test]
    fn picks_highest_leo() {
        let reports = vec![
            report(1, 50, 3, true),
            report(2, 100, 3, true),
            report(3, 80, 3, true),
        ];
        assert_eq!(elect_recovery_leader(&reports), Some(2));
    }

    #[test]
    fn tie_leo_breaks_on_leader_epoch() {
        let reports = vec![report(1, 100, 2, true), report(2, 100, 5, true)];
        assert_eq!(elect_recovery_leader(&reports), Some(2));
    }

    #[test]
    fn unavailable_excluded() {
        let reports = vec![report(1, 100, 3, false), report(2, 50, 3, true)];
        assert_eq!(elect_recovery_leader(&reports), Some(2));
    }

    #[test]
    fn none_available_returns_none() {
        let reports = vec![report(1, 100, 3, false), report(2, 50, 3, false)];
        assert_eq!(elect_recovery_leader(&reports), None);
    }

    #[test]
    fn empty_returns_none() {
        assert_eq!(elect_recovery_leader(&[]), None);
    }
}
