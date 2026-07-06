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

use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::{loop_select_ticket, now_second},
};
use common_config::broker::broker_config;
use grpc_clients::{meta::storage::call::delete_segment, pool::ClientPool};
use protocol::meta::meta_service_journal::{DeleteSegmentRaw, DeleteSegmentRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::core::cache::StorageCacheManager;
use crate::core::segment::{delete_local_segment, list_segments};
use crate::filesegment::SegmentIdentity;

pub async fn start_segment_expire_thread(
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<StorageCacheManager>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        scan_and_delete_segment0(&client_pool, &cache_manager).await?;
        Ok(())
    };
    loop_select_ticket(ac_fn, 600000, stop_sx).await;
}

pub async fn start_orphan_clean_thread(
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        if let Err(e) =
            scan_and_clean_orphan_segments(&client_pool, &cache_manager, &rocksdb_engine_handler)
                .await
        {
            warn!("orphan segment cleanup failed: {}", e);
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, 3600000, stop_sx).await;
}

async fn scan_and_delete_segment0(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
) -> Result<(), CommonError> {
    let conf = broker_config();
    let broker_id = conf.broker_id;
    let current_time = now_second();
    let mut segment_list = Vec::new();

    for shard_entry in cache_manager.shards.iter() {
        let shard_name = shard_entry.key();
        let retention_sec = shard_entry.value().config.retention_sec;
        let earliest_timestamp = current_time.saturating_sub(retention_sec) as i64;

        let Some(index) = cache_manager.get_offset_index(shard_name) else {
            continue;
        };

        for seq in index.expired_head_seqs(earliest_timestamp) {
            let is_leader = cache_manager
                .segments
                .get(shard_name)
                .and_then(|m| m.get(&seq).map(|s| s.leader == broker_id))
                .unwrap_or(false);

            if is_leader {
                segment_list.push(DeleteSegmentRaw {
                    shard_name: shard_name.clone(),
                    segment: seq,
                });
            }
        }
    }

    if segment_list.is_empty() {
        return Ok(());
    }

    let request = DeleteSegmentRequest { segment_list };
    delete_segment(client_pool, &conf.get_meta_service_addr(), request).await?;
    Ok(())
}

async fn scan_and_clean_orphan_segments(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<(), CommonError> {
    let conf = broker_config();
    let broker_id = conf.broker_id;

    let meta_segments: HashSet<(String, u32)> = list_segments(client_pool)
        .await
        .map_err(|e| CommonError::CommonError(e.to_string()))?
        .into_iter()
        .map(|s| (s.shard_name, s.segment_seq))
        .collect();

    let orphans = collect_follower_orphans(cache_manager, broker_id, &meta_segments);
    for segment_iden in &orphans {
        match delete_local_segment(cache_manager, rocksdb_engine_handler, segment_iden).await {
            Ok(()) => info!("Deleted orphan follower segment {}", segment_iden.name()),
            Err(e) => warn!(
                "Failed to delete orphan follower segment {}: {}",
                segment_iden.name(),
                e
            ),
        }
    }
    Ok(())
}

fn collect_follower_orphans(
    cache_manager: &Arc<StorageCacheManager>,
    broker_id: u64,
    meta_segments: &HashSet<(String, u32)>,
) -> Vec<SegmentIdentity> {
    let mut orphans = Vec::new();
    for raw in cache_manager.segments.iter() {
        let shard_name = raw.key();
        for segment in raw.value().iter() {
            if segment.leader == broker_id {
                continue;
            }
            if !segment.replicas.iter().any(|r| r.node_id == broker_id) {
                continue;
            }
            if !meta_segments.contains(&(shard_name.clone(), segment.segment_seq)) {
                orphans.push(SegmentIdentity::new(shard_name, segment.segment_seq));
            }
        }
    }
    orphans
}

#[cfg(test)]
mod tests {
    use super::collect_follower_orphans;
    use crate::core::test_tool::test_build_memory_engine;
    use metadata_struct::storage::segment::{EngineSegment, Replica};
    use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
    use std::collections::HashSet;
    use std::sync::Arc;

    fn make_segment(shard_name: &str, seq: u32, leader: u64, replicas: &[u64]) -> EngineSegment {
        EngineSegment {
            shard_name: shard_name.to_string(),
            segment_seq: seq,
            leader,
            replicas: replicas
                .iter()
                .map(|&id| Replica {
                    node_id: id,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn meta_set(items: &[(&str, u32)]) -> HashSet<(String, u32)> {
        items.iter().map(|(s, n)| (s.to_string(), *n)).collect()
    }

    #[tokio::test]
    async fn orphan_detection_skips_leader_segments() {
        let engine = Arc::new(test_build_memory_engine());
        let cm = engine.cache_manager.clone();
        let broker_id = 1_u64;

        cm.set_shard(EngineShard {
            shard_name: "s".to_string(),
            config: EngineShardConfig {
                storage_type: common_config::storage::StorageType::EngineSegment,
                ..Default::default()
            },
            ..Default::default()
        });
        cm.set_segment(&make_segment("s", 0, broker_id, &[broker_id, 2, 3]));

        let orphans = collect_follower_orphans(&cm, broker_id, &meta_set(&[]));
        assert!(
            orphans.is_empty(),
            "leader-owned segments must not be orphaned"
        );
    }

    #[tokio::test]
    async fn orphan_detection_returns_follower_not_in_meta() {
        let engine = Arc::new(test_build_memory_engine());
        let cm = engine.cache_manager.clone();
        let broker_id = 2_u64;
        let leader_id = 1_u64;

        cm.set_shard(EngineShard {
            shard_name: "s".to_string(),
            config: EngineShardConfig {
                storage_type: common_config::storage::StorageType::EngineSegment,
                ..Default::default()
            },
            ..Default::default()
        });
        cm.set_segment(&make_segment("s", 0, leader_id, &[leader_id, broker_id]));
        cm.set_segment(&make_segment("s", 1, leader_id, &[leader_id, broker_id]));

        let meta = meta_set(&[("s", 0)]);
        let orphans = collect_follower_orphans(&cm, broker_id, &meta);

        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].shard_name, "s");
        assert_eq!(orphans[0].segment, 1);
    }

    #[tokio::test]
    async fn orphan_detection_skips_non_replica_segments() {
        let engine = Arc::new(test_build_memory_engine());
        let cm = engine.cache_manager.clone();
        let broker_id = 2_u64;
        let leader_id = 1_u64;

        cm.set_shard(EngineShard {
            shard_name: "s".to_string(),
            config: EngineShardConfig {
                storage_type: common_config::storage::StorageType::EngineSegment,
                ..Default::default()
            },
            ..Default::default()
        });
        cm.set_segment(&make_segment("s", 0, leader_id, &[leader_id, 3]));

        let orphans = collect_follower_orphans(&cm, broker_id, &meta_set(&[]));
        assert!(
            orphans.is_empty(),
            "segments where this node is not a replica must be skipped"
        );
    }
}
