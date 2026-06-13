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
use crate::storage::common::node::NodeStorage;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_config::broker::broker_config;
use metadata_struct::storage::segment::{EngineSegment, SegmentStatus};
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

pub async fn start_segment_leader_rebalance_thread(
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
    call_manager: Arc<NodeCallManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_send: broadcast::Sender<bool>,
) {
    let interval = broker_config()
        .meta_runtime
        .segment_leader_rebalance_interval_ms;
    let ac_fn = async || -> ResultCommonError {
        if raft_manager.is_metadata_leader() {
            rebalance_once(
                &raft_manager,
                &cache_manager,
                &call_manager,
                &rocksdb_engine_handler,
            )
            .await;
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, interval, &stop_send).await;
}

async fn rebalance_once(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) {
    // Only the active segment carries write/replication load; sealed segments are
    // read-only and Unavailable ones are the recovery path's job.
    let candidates: Vec<EngineSegment> = cache_manager
        .shard_list
        .iter()
        .filter_map(|shard| cache_manager.get_segment(&shard.shard_name, shard.active_segment_seq))
        .filter(|seg| should_rebalance(cache_manager, seg))
        .collect();

    let max_moves = broker_config()
        .meta_runtime
        .segment_leader_rebalance_max_moves;
    let mut moved = 0u32;
    for segment in candidates {
        if moved >= max_moves {
            break;
        }
        match switch_to_preferred(
            raft_manager,
            cache_manager,
            call_manager,
            rocksdb_engine_handler,
            &segment,
        )
        .await
        {
            Ok(true) => moved += 1,
            Ok(false) => {}
            Err(e) => warn!(
                "segment leader rebalance failed for {}/{}: {}",
                segment.shard_name, segment.segment_seq, e
            ),
        }
    }

    if moved > 0 {
        info!("segment leader rebalance moved {moved} leaders back to their preferred replica");
    }
}

/// The active segment should move its leadership back to the preferred replica
/// (replicas[0], the leader chosen at creation) when that replica is alive and
/// in-sync — restoring the balanced placement that failover/recovery moved away.
fn should_rebalance(cache_manager: &Arc<MetaCacheManager>, seg: &EngineSegment) -> bool {
    if seg.status != SegmentStatus::Write {
        return false;
    }
    let Some(preferred) = seg.replicas.first().map(|r| r.node_id) else {
        return false;
    };
    preferred != seg.leader
        && seg.isr.contains(&preferred)
        && cache_manager.get_broker_node(preferred).is_some()
}

async fn switch_to_preferred(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment: &EngineSegment,
) -> Result<bool, String> {
    // Re-read and re-validate against the latest state (it may have changed since
    // the candidate snapshot was taken).
    let current = match cache_manager.get_segment(&segment.shard_name, segment.segment_seq) {
        Some(s) => s,
        None => return Ok(false),
    };
    if !should_rebalance(cache_manager, &current) {
        return Ok(false);
    }
    let Some(preferred) = current.replicas.first().map(|r| r.node_id) else {
        return Ok(false);
    };

    // ISR stays intact: preferred is already in-sync, the old leader stays a
    // follower in the ISR. Only the leadership term changes.
    let node_storage = NodeStorage::new(rocksdb_engine_handler.clone());
    let preferred_broker_epoch = node_storage
        .get_broker_epoch(preferred)
        .map_err(|e| e.to_string())?;

    let mut new_segment = current.clone();
    new_segment.leader = preferred;
    new_segment.leader_epoch += 1;
    new_segment.segment_epoch += 1;
    new_segment.leader_broker_epoch = preferred_broker_epoch;

    sync_save_segment_info(raft_manager, &new_segment)
        .await
        .map_err(|e| e.to_string())?;

    // Cross-node CAS check: verify our write won (another meta node may have raced us).
    if let Some(after) = cache_manager.get_segment(&current.shard_name, current.segment_seq) {
        if after.leader != preferred {
            return Ok(false);
        }
    }

    send_notify_by_set_segment(call_manager, new_segment.clone())
        .await
        .map_err(|e| e.to_string())?;

    info!(
        "segment leader rebalance: {}/{} leader {} -> preferred {}",
        current.shard_name, current.segment_seq, current.leader, preferred
    );
    Ok(true)
}
