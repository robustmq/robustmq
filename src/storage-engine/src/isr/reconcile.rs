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

use crate::core::cache::StorageCacheManager;
use crate::filesegment::SegmentIdentity;
use crate::isr::fetcher_manager::ReplicaFetcherManager;
use crate::isr::role::apply_leader_and_isr;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_config::broker::broker_config;
use grpc_clients::meta::storage::call::list_segment;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::EngineSegment;
use protocol::meta::meta_service_journal::{ListSegmentFilter, ListSegmentRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::warn;

pub async fn start_metadata_reconcile_thread(
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<StorageCacheManager>,
    fetcher_manager: Arc<ReplicaFetcherManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let interval = broker_config()
        .storage_runtime
        .metadata_reconcile_interval_ms;
    let ac_fn = async || -> ResultCommonError {
        let conf = broker_config();
        let addrs = conf.get_meta_service_addr();

        reconcile_urgent(
            &client_pool,
            &cache_manager,
            &fetcher_manager,
            &rocksdb_engine_handler,
            &addrs,
        )
        .await;

        reconcile_active(
            &client_pool,
            &cache_manager,
            &fetcher_manager,
            &rocksdb_engine_handler,
            &addrs,
        )
        .await;
        Ok(())
    };
    loop_select_ticket(ac_fn, interval, stop_sx).await;
}

/// Handle segments flagged for immediate reconcile (e.g. on UnknownLeaderEpoch from fetch).
async fn reconcile_urgent(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    addrs: &[String],
) {
    let urgent = cache_manager.take_reconcile_needed();
    if urgent.is_empty() {
        return;
    }

    let known: std::collections::HashMap<(String, u32), u32> = urgent
        .iter()
        .map(|(shard, seq)| {
            (
                (shard.clone(), *seq),
                local_epoch(cache_manager, shard, *seq),
            )
        })
        .collect();

    let filters: Vec<ListSegmentFilter> = urgent
        .into_iter()
        .map(|(shard, seq)| ListSegmentFilter {
            shard_name: shard,
            segment: seq as i32,
        })
        .collect();

    let segments = batch_fetch_segments(client_pool, addrs, filters).await;
    for segment in segments {
        let known_epoch = known
            .get(&(segment.shard_name.clone(), segment.segment_seq))
            .copied()
            .unwrap_or(0);
        if segment.segment_epoch > known_epoch {
            apply_segment(
                cache_manager,
                rocksdb_engine_handler,
                fetcher_manager,
                segment,
            )
            .await;
        }
    }
}

/// Active reconcile: only check the active segment per shard.
/// Bounds comparison to O(shards) rather than O(all segments).
async fn reconcile_active(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    addrs: &[String],
) {
    let locals = collect_active_locals(cache_manager);
    if locals.is_empty() {
        return;
    }

    let known: std::collections::HashMap<(String, u32), u32> = locals
        .iter()
        .map(|(shard, seq, epoch)| ((shard.clone(), *seq), *epoch))
        .collect();

    let filters: Vec<ListSegmentFilter> = locals
        .into_iter()
        .map(|(shard, seq, _)| ListSegmentFilter {
            shard_name: shard,
            segment: seq as i32,
        })
        .collect();

    let segments = batch_fetch_segments(client_pool, addrs, filters).await;
    for segment in segments {
        let known_epoch = known
            .get(&(segment.shard_name.clone(), segment.segment_seq))
            .copied()
            .unwrap_or(0);
        if segment.segment_epoch > known_epoch {
            apply_segment(
                cache_manager,
                rocksdb_engine_handler,
                fetcher_manager,
                segment,
            )
            .await;
        }
    }
}

/// Returns (shard_name, active_segment_seq, local_epoch) for every shard.
fn collect_active_locals(cache_manager: &StorageCacheManager) -> Vec<(String, u32, u32)> {
    cache_manager
        .shards
        .iter()
        .map(|entry| {
            let shard_name = entry.key().clone();
            let active_seq = entry.value().active_segment_seq;
            let epoch = local_epoch(cache_manager, &shard_name, active_seq);
            (shard_name, active_seq, epoch)
        })
        .collect()
}

/// Batch-fetches the listed (shard, segment) pairs in a single RPC, returns decoded segments.
async fn batch_fetch_segments(
    client_pool: &Arc<ClientPool>,
    addrs: &[String],
    filters: Vec<ListSegmentFilter>,
) -> Vec<EngineSegment> {
    let req = ListSegmentRequest {
        shard_name: String::new(),
        segment: -1,
        filters,
    };

    let mut stream = match list_segment(client_pool, addrs, req).await {
        Ok(s) => s,
        Err(e) => {
            warn!("reconcile batch list_segment: {}", e);
            return Vec::new();
        }
    };

    let mut result = Vec::new();
    loop {
        match stream.message().await {
            Ok(Some(reply)) => match EngineSegment::decode(&reply.segment) {
                Ok(seg) => result.push(seg),
                Err(e) => warn!("reconcile decode: {}", e),
            },
            Ok(None) => break,
            Err(e) => {
                warn!("reconcile stream: {}", e);
                break;
            }
        }
    }
    result
}

/// Updates cache and applies role/ISR transition for the segment.
async fn apply_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    segment: EngineSegment,
) {
    cache_manager.set_segment(&segment);
    if let Err(e) = apply_leader_and_isr(
        cache_manager,
        rocksdb_engine_handler,
        fetcher_manager,
        &segment,
    )
    .await
    {
        warn!(
            "reconcile apply {}/{}: {}",
            segment.shard_name, segment.segment_seq, e
        );
    }
}

fn local_epoch(cache_manager: &StorageCacheManager, shard: &str, segment_seq: u32) -> u32 {
    let ident = SegmentIdentity {
        shard_name: shard.to_string(),
        segment: segment_seq,
    };
    cache_manager
        .get_segment(&ident)
        .map(|s| s.segment_epoch)
        .unwrap_or(0)
}
