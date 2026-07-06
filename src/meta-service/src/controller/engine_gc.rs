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
use crate::core::notify::{send_notify_by_delete_segment, send_notify_by_delete_shard};
use crate::core::segment::delete_segment_by_real;
use crate::core::shard::delete_shard_by_real;
use crate::raft::manager::MultiRaftManager;
use common_base::error::common::CommonError;
use common_base::tools::loop_select_ticket;
use grpc_clients::broker::common::call::broker_get_shard_segment_delete_status;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::EngineSegment;
use node_call::NodeCallManager;
use protocol::broker::broker::{GetShardSegmentDeleteStatusRequest, ShardSegmentStatusItem};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::warn;

pub async fn start_engine_delete_gc_thread(
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
    node_call_manager: Arc<NodeCallManager>,
    client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = || {
        let raft_manager = raft_manager.clone();
        let cache_manager = cache_manager.clone();
        let node_call_manager = node_call_manager.clone();
        let client_pool = client_pool.clone();
        async move {
            if let Err(e) = gc_shard(
                &raft_manager,
                &cache_manager,
                &node_call_manager,
                &client_pool,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            };

            if let Err(e) = gc_segment(
                &raft_manager,
                &node_call_manager,
                &cache_manager,
                &client_pool,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }

            Ok(())
        }
    };
    loop_select_ticket(ac_fn, 5000, &stop_send).await;
}

async fn gc_shard(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    node_call_manager: &Arc<NodeCallManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), MetaServiceError> {
    for shard_name in cache_manager.get_wait_delete_shard_list() {
        let addrs = shard_replica_addrs(cache_manager, &shard_name);
        if addrs.is_empty() {
            warn!(
                "shard {} has no known replica nodes, skipping GC check this round",
                shard_name
            );
            continue;
        }
        let item = ShardSegmentStatusItem {
            shard_name: shard_name.clone(),
            segment_seq: None,
        };

        if check_deleted(client_pool, &addrs, item).await {
            if let Err(e) = delete_shard_by_real(cache_manager, raft_manager, &shard_name).await {
                warn!("delete shard {} failed: {}", shard_name, e);
            }
        } else if let Some(shard) = cache_manager.shard_list.get(&shard_name) {
            if let Err(e) = send_notify_by_delete_shard(node_call_manager, shard.clone()).await {
                warn!("Failed to notify delete shard {}: {}", shard_name, e);
            }
        }
    }

    Ok(())
}

async fn gc_segment(
    raft_manager: &Arc<MultiRaftManager>,
    node_call_manager: &Arc<NodeCallManager>,
    cache_manager: &Arc<MetaCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), MetaServiceError> {
    for segment in cache_manager.get_wait_delete_segment_list() {
        if cache_manager
            .get_segment(&segment.shard_name, segment.segment_seq)
            .is_none()
        {
            cache_manager.remove_wait_delete_segment(&segment);
            continue;
        }

        let addrs = segment_replica_addrs(cache_manager, &segment);
        if addrs.is_empty() {
            warn!(
                "segment {}/{} has no known replica nodes, skipping GC check this round",
                segment.shard_name, segment.segment_seq
            );
            continue;
        }
        let item = ShardSegmentStatusItem {
            shard_name: segment.shard_name.clone(),
            segment_seq: Some(segment.segment_seq),
        };

        if check_deleted(client_pool, &addrs, item).await {
            if let Err(e) = delete_segment_by_real(cache_manager, raft_manager, &segment).await {
                warn!(
                    "delete segment {}/{} failed: {}",
                    segment.shard_name, segment.segment_seq, e
                );
            } else {
                cache_manager.remove_wait_delete_segment(&segment);
            }
        } else if let Err(e) =
            send_notify_by_delete_segment(node_call_manager, segment.clone()).await
        {
            warn!(
                "Failed to notify delete segment {}/{}: {}",
                segment.shard_name, segment.segment_seq, e
            );
        }
    }

    Ok(())
}

async fn check_deleted(
    client_pool: &Arc<ClientPool>,
    addrs: &[String],
    item: ShardSegmentStatusItem,
) -> bool {
    if addrs.is_empty() {
        return false;
    }

    let req = GetShardSegmentDeleteStatusRequest { items: vec![item] };

    for addr in addrs {
        match broker_get_shard_segment_delete_status(client_pool, &[addr.as_str()], req.clone())
            .await
        {
            Ok(reply) => {
                if reply.results.iter().any(|r| !r.deleted) {
                    return false;
                }
            }
            Err(e) => {
                warn!("Failed to get delete status from broker {}: {}", addr, e);
                return false;
            }
        }
    }

    true
}

fn shard_replica_addrs(cache_manager: &Arc<MetaCacheManager>, shard_name: &str) -> Vec<String> {
    let mut node_ids: Vec<u64> = cache_manager
        .get_segment_list_by_shard(shard_name)
        .iter()
        .flat_map(|seg| seg.replicas.iter().map(|r| r.node_id))
        .collect();
    node_ids.sort_unstable();
    node_ids.dedup();
    node_ids_to_addrs(cache_manager, &node_ids)
}

fn segment_replica_addrs(
    cache_manager: &Arc<MetaCacheManager>,
    segment: &EngineSegment,
) -> Vec<String> {
    let node_ids: Vec<u64> = segment.replicas.iter().map(|r| r.node_id).collect();
    node_ids_to_addrs(cache_manager, &node_ids)
}

fn node_ids_to_addrs(cache_manager: &Arc<MetaCacheManager>, node_ids: &[u64]) -> Vec<String> {
    node_ids
        .iter()
        .filter_map(|id| cache_manager.get_broker_node(*id))
        .map(|n| n.grpc_addr)
        .collect()
}
