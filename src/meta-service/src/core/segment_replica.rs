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
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use metadata_struct::storage::segment::{EngineSegment, Replica, SegmentStatus};
use metadata_struct::storage::shard::EngineShard;
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

impl MetaCacheManager {
    /// Per-node (replica_count, leader_count) used for balanced placement.
    /// Lazily initialized from a single full scan on first call; thereafter
    /// maintained incrementally by `set_segment`/`remove_segment`.
    pub fn node_loads(&self) -> (HashMap<u64, u64>, HashMap<u64, u64>) {
        if !self.node_load.initialized.load(Ordering::Acquire) {
            let _guard = self.node_load.init_lock.lock().unwrap();
            if !self.node_load.initialized.load(Ordering::Acquire) {
                self.node_load.replica_count.clear();
                self.node_load.leader_count.clear();
                for shard in self.segment_list.iter() {
                    for segment in shard.iter() {
                        for replica in &segment.replicas {
                            *self
                                .node_load
                                .replica_count
                                .entry(replica.node_id)
                                .or_insert(0) += 1;
                        }
                        *self
                            .node_load
                            .leader_count
                            .entry(segment.leader)
                            .or_insert(0) += 1;
                    }
                }
                self.node_load.initialized.store(true, Ordering::Release);
            }
        }
        let replica = self
            .node_load
            .replica_count
            .iter()
            .map(|e| (*e.key(), *e.value()))
            .collect();
        let leader = self
            .node_load
            .leader_count
            .iter()
            .map(|e| (*e.key(), *e.value()))
            .collect();
        (replica, leader)
    }
}

/// Build the initial replica/leader placement for a new segment.
///
/// Replicas are placed on the least replica-loaded nodes and the leader is the
/// least leader-loaded among them, so both replica and leadership load spread
/// evenly across the cluster (instead of the previous random placement). The
/// elected leader is kept at `replicas[0]` so it matches the preferred-replica
/// that the leader-rebalance controller tries to hold leadership on.
pub async fn build_segment(
    shard_info: &EngineShard,
    cache_manager: &Arc<MetaCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_no: u32,
) -> Result<EngineSegment, MetaServiceError> {
    if let Some(segment) = cache_manager.get_segment(&shard_info.shard_name, segment_no) {
        return Ok(segment);
    }

    let alive: Vec<u64> = cache_manager
        .get_engine_node_list()
        .iter()
        .map(|n| n.node_id)
        .collect();

    let target_replicas = effective_replica_num(
        shard_info.config.is_inner_topic,
        shard_info.config.replica_num as usize,
        alive.len(),
    )?;

    let (replica_load, leader_load) = cache_manager.node_loads();

    let chosen = select_least_loaded(&alive, &replica_load, target_replicas);
    let leader = pick_leader(&chosen, &leader_load)?;
    let ordered = order_leader_first(chosen, leader);

    let mut replicas = Vec::with_capacity(ordered.len());
    for (seq, node_id) in ordered.iter().enumerate() {
        let fold = calc_node_fold(cache_manager, *node_id)?;
        replicas.push(Replica {
            replica_seq: seq as u64,
            node_id: *node_id,
            fold,
        });
    }

    let isr: Vec<u64> = replicas.iter().map(|r| r.node_id).collect();
    let node_storage = NodeStorage::new(rocksdb_engine_handler.clone());
    let leader_broker_epoch = node_storage.get_broker_epoch(leader)?;

    Ok(EngineSegment {
        shard_name: shard_info.shard_name.clone(),
        leader_epoch: 0,
        status: SegmentStatus::Write,
        segment_seq: segment_no,
        leader,
        replicas,
        isr,
        leader_broker_epoch,
        ..Default::default()
    })
}

/// How many replicas to actually place now. An inner/system topic may start
/// under-replicated (a background task tops it up to `replica_num` later) as
/// long as at least one engine node is alive; a regular topic requires enough
/// live nodes for its full replica set.
fn effective_replica_num(
    is_inner_topic: bool,
    replica_num: usize,
    alive: usize,
) -> Result<usize, MetaServiceError> {
    if is_inner_topic {
        if alive == 0 {
            return Err(MetaServiceError::NotEnoughEngineNodes(
                "CreateSegment".to_string(),
                1,
                0,
            ));
        }
        Ok(replica_num.min(alive).max(1))
    } else {
        if alive < replica_num {
            return Err(MetaServiceError::NotEnoughEngineNodes(
                "CreateSegment".to_string(),
                replica_num as u32,
                alive as u32,
            ));
        }
        Ok(replica_num)
    }
}

/// Pick the `count` least-loaded nodes, breaking ties by node id (deterministic).
fn select_least_loaded(candidates: &[u64], load: &HashMap<u64, u64>, count: usize) -> Vec<u64> {
    let mut sorted = candidates.to_vec();
    sorted.sort_by_key(|id| (*load.get(id).unwrap_or(&0), *id));
    sorted.truncate(count);
    sorted
}

/// Among `nodes`, pick the least leader-loaded, breaking ties by node id.
fn pick_leader(nodes: &[u64], load: &HashMap<u64, u64>) -> Result<u64, MetaServiceError> {
    nodes
        .iter()
        .copied()
        .min_by_key(|id| (*load.get(id).unwrap_or(&0), *id))
        .ok_or_else(|| MetaServiceError::CommonError("no candidate node for leader".to_string()))
}

/// Place `leader` at index 0, keeping the other nodes' relative order.
fn order_leader_first(mut nodes: Vec<u64>, leader: u64) -> Vec<u64> {
    nodes.retain(|&n| n != leader);
    let mut ordered = Vec::with_capacity(nodes.len() + 1);
    ordered.push(leader);
    ordered.extend(nodes);
    ordered
}

// How often the leader scans inner topics to top up under-replicated segments.
const INNER_TOPIC_REPLICA_FILL_INTERVAL_MS: u64 = 30_000;

/// Background thread (meta leader only): periodically scans inner/system topics
/// and tops up any segment whose replica count is below `replica_num`, placing
/// the extra replicas on the least-loaded live nodes. New replicas are added to
/// the replica set only (not the ISR) — they catch up from the leader and the
/// ISR maintainer admits them once in sync. If no node is available to take a
/// replica it is left as-is (no error) and retried next tick.
pub async fn start_inner_topic_replica_fill_thread(
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
    call_manager: Arc<NodeCallManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        if raft_manager.is_metadata_leader() {
            fill_inner_topic_replicas_once(&raft_manager, &cache_manager, &call_manager).await;
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, INNER_TOPIC_REPLICA_FILL_INTERVAL_MS, &stop_send).await;
}

async fn fill_inner_topic_replicas_once(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    call_manager: &Arc<NodeCallManager>,
) {
    let inner_shards: Vec<EngineShard> = cache_manager
        .shard_list
        .iter()
        .filter(|s| s.config.is_inner_topic)
        .map(|s| s.clone())
        .collect();
    if inner_shards.is_empty() {
        return;
    }

    let alive: Vec<u64> = cache_manager
        .get_engine_node_list()
        .iter()
        .map(|n| n.node_id)
        .collect();
    if alive.is_empty() {
        return;
    }

    // Snapshot of current load, updated locally as replicas are added so
    // successive fills within the same tick keep spreading load.
    let (mut load, _) = cache_manager.node_loads();

    let mut filled = 0u32;
    for shard in inner_shards {
        let target = shard.config.replica_num as usize;
        for segment in cache_manager.get_segment_list_by_shard(&shard.shard_name) {
            if segment.replicas.len() >= target {
                continue;
            }
            let existing: HashSet<u64> = segment.replicas.iter().map(|r| r.node_id).collect();
            let candidates: Vec<u64> = alive
                .iter()
                .copied()
                .filter(|n| !existing.contains(n))
                .collect();
            let need = target - segment.replicas.len();
            let to_add = select_least_loaded(&candidates, &load, need);
            if to_add.is_empty() {
                continue;
            }

            let new_segment =
                match build_segment_with_added_replicas(cache_manager, &segment, &to_add) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(
                            "inner topic replica fill {}/{}: build failed: {}",
                            segment.shard_name, segment.segment_seq, e
                        );
                        continue;
                    }
                };

            if let Err(e) = sync_save_segment_info(raft_manager, &new_segment).await {
                warn!(
                    "inner topic replica fill {}/{}: save failed: {}",
                    segment.shard_name, segment.segment_seq, e
                );
                continue;
            }
            if let Err(e) = send_notify_by_set_segment(call_manager, new_segment).await {
                warn!(
                    "inner topic replica fill {}/{}: notify failed: {}",
                    segment.shard_name, segment.segment_seq, e
                );
                continue;
            }
            for node_id in &to_add {
                *load.entry(*node_id).or_insert(0) += 1;
            }
            filled += 1;
        }
    }

    if filled > 0 {
        info!(
            "inner topic replica fill: topped up {} segment(s) toward their replica_num",
            filled
        );
    }
}

/// Append `to_add` as replica-set-only members (not ISR). Bumps segment_epoch
/// since the replica set changed.
fn build_segment_with_added_replicas(
    cache_manager: &Arc<MetaCacheManager>,
    segment: &EngineSegment,
    to_add: &[u64],
) -> Result<EngineSegment, MetaServiceError> {
    let mut new_segment = segment.clone();
    let base_seq = new_segment
        .replicas
        .iter()
        .map(|r| r.replica_seq)
        .max()
        .map_or(0, |m| m + 1);
    for (offset, node_id) in to_add.iter().enumerate() {
        let fold = calc_node_fold(cache_manager, *node_id)?;
        new_segment.replicas.push(Replica {
            replica_seq: base_seq + offset as u64,
            node_id: *node_id,
            fold,
        });
    }
    new_segment.segment_epoch += 1;
    Ok(new_segment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn load(pairs: &[(u64, u64)]) -> HashMap<u64, u64> {
        pairs.iter().copied().collect()
    }

    fn seg(seq: u32, leader: u64, replicas: &[u64]) -> EngineSegment {
        EngineSegment {
            shard_name: "s".to_string(),
            segment_seq: seq,
            leader,
            replicas: replicas
                .iter()
                .map(|id| Replica {
                    replica_seq: 0,
                    node_id: *id,
                    fold: String::new(),
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn node_load_tracks_set_update_remove() {
        let cache = MetaCacheManager::new(test_rocksdb_instance());
        // Initialize the cache (empty) so incremental maintenance is active.
        cache.node_loads();

        cache.set_segment(seg(0, 1, &[1, 2, 3]));
        let (replica, leader) = cache.node_loads();
        assert_eq!((replica[&1], replica[&2], replica[&3]), (1, 1, 1));
        assert_eq!(leader[&1], 1);

        // Update: leader 1->2, replicas {1,2,3}->{2,3,4}.
        cache.set_segment(seg(0, 2, &[2, 3, 4]));
        let (replica, leader) = cache.node_loads();
        assert_eq!(replica[&1], 0);
        assert_eq!((replica[&2], replica[&3], replica[&4]), (1, 1, 1));
        assert_eq!((leader[&1], leader[&2]), (0, 1));

        // Removing the node drops its entries entirely.
        cache.remove_broker_node(2);
        let (replica, leader) = cache.node_loads();
        assert!(!replica.contains_key(&2));
        assert!(!leader.contains_key(&2));

        cache.remove_segment("s", 0);
        let (replica, leader) = cache.node_loads();
        assert_eq!(replica[&3], 0);
        assert_eq!(replica[&4], 0);
        assert!(leader.values().all(|&c| c == 0));
    }

    #[test]
    fn select_least_loaded_picks_lowest_and_breaks_ties_by_id() {
        let candidates = [1, 2, 3, 4];
        let l = load(&[(1, 5), (2, 0), (3, 0), (4, 2)]);
        // counts: 2->0, 3->0, 4->2, 1->5; pick 2 least → [2, 3] (tie 2,3 → by id).
        assert_eq!(select_least_loaded(&candidates, &l, 2), vec![2, 3]);
    }

    #[test]
    fn select_least_loaded_truncates_to_available() {
        let candidates = [7, 9];
        let l = load(&[(7, 0), (9, 0)]);
        assert_eq!(select_least_loaded(&candidates, &l, 5), vec![7, 9]);
    }

    #[test]
    fn pick_leader_is_least_leader_loaded() {
        let l = load(&[(2, 3), (3, 1), (5, 1)]);
        // 3 and 5 tie at 1 → lowest id 3.
        assert_eq!(pick_leader(&[2, 3, 5], &l).unwrap(), 3);
    }

    #[test]
    fn order_leader_first_moves_leader_to_front() {
        assert_eq!(order_leader_first(vec![2, 3, 5], 5), vec![5, 2, 3]);
        assert_eq!(order_leader_first(vec![2, 3, 5], 2), vec![2, 3, 5]);
    }

    #[test]
    fn inner_topic_allows_under_replication_but_needs_one_node() {
        // inner topic, want 3, only 1 alive → place 1 now.
        assert_eq!(effective_replica_num(true, 3, 1).unwrap(), 1);
        // inner topic, enough nodes → full replica set.
        assert_eq!(effective_replica_num(true, 3, 5).unwrap(), 3);
        // inner topic, no live node → error.
        assert!(effective_replica_num(true, 3, 0).is_err());
    }

    #[test]
    fn regular_topic_requires_enough_nodes() {
        assert_eq!(effective_replica_num(false, 2, 3).unwrap(), 2);
        assert!(effective_replica_num(false, 2, 1).is_err());
    }
}
