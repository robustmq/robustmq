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

use crate::core::notify::send_notify_by_delete_group_offset;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::common::offset::OffsetStorage;
use crate::storage::common::share_group::ShareGroupStorage;
use broker_core::cache::NodeCacheManager;
use bytes::Bytes;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use node_call::NodeCallManager;
use prost::Message as _;
use protocol::meta::meta_service_common::DeleteShareGroupRequest;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

// Scan every 10 seconds
const GROUP_GC_INTERVAL_MS: u64 = 10 * 1000;

pub async fn start_group_gc_thread(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    raft_manager: Arc<MultiRaftManager>,
    node_call_manager: Arc<NodeCallManager>,
    node_cache: Arc<NodeCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        let expire_sec = node_cache
            .get_cluster_config()
            .meta_runtime
            .group_offset_expire_sec;
        if let Err(e) = gc_expired_groups(
            &rocksdb_engine_handler,
            &raft_manager,
            &node_call_manager,
            expire_sec,
        )
        .await
        {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, GROUP_GC_INTERVAL_MS, &stop_send).await;
}

async fn gc_expired_groups(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    node_call_manager: &Arc<NodeCallManager>,
    expire_sec: u64,
) -> Result<(), CommonError> {
    let storage = OffsetStorage::new(rocksdb_engine_handler.clone());
    let all_offsets = storage.list_all()?;

    // Collect the latest timestamp per (tenant, group).
    // A group is only expired when ALL its shards haven't been committed for expire_sec.
    let mut group_latest: HashMap<(String, String), u64> = HashMap::new();
    for offset in &all_offsets {
        let key = (offset.tenant.clone(), offset.group.clone());
        let entry = group_latest.entry(key).or_insert(0);
        if offset.timestamp > *entry {
            *entry = offset.timestamp;
        }
    }

    let now = now_second();
    for ((tenant, group), latest_ts) in group_latest {
        if now.saturating_sub(latest_ts) < expire_sec {
            continue;
        }

        // Delete all shard offset records for this group via raft.
        let delete_req = DeleteShareGroupRequest {
            tenant: tenant.clone(),
            group: group.clone(),
        };
        let data = StorageData::new(
            StorageDataType::OffsetDelete,
            Bytes::from(delete_req.encode_to_vec()),
        );
        if let Err(e) = raft_manager.write_data(&group, data).await {
            warn!(
                "Failed to delete offset records via raft: tenant={}, group={}, error={}",
                tenant, group, e
            );
            continue;
        }

        // Delete share group via raft only if it exists.
        let share_group_storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
        match share_group_storage.get(&tenant, &group) {
            Ok(Some(leader)) => match leader.encode() {
                Ok(encoded) => {
                    let data = StorageData::new(
                        StorageDataType::MqttDeleteGroupLeader,
                        Bytes::from(encoded),
                    );
                    if let Err(e) = raft_manager.write_data(&group, data).await {
                        warn!(
                            "Failed to delete share group via raft: tenant={}, group={}, error={}",
                            tenant, group, e
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to encode ShareGroupLeader for deletion: tenant={}, group={}, error={}",
                        tenant, group, e
                    );
                }
            },
            Ok(None) => {}
            Err(e) => {
                warn!(
                    "Failed to check share group existence: tenant={}, group={}, error={}",
                    tenant, group, e
                );
            }
        }

        // Notify all broker nodes to clean up their in-memory group state.
        if let Err(e) = send_notify_by_delete_group_offset(node_call_manager, &tenant, &group).await
        {
            warn!(
                "Failed to notify brokers to delete group: tenant={}, group={}, error={}",
                tenant, group, e
            );
        }

        info!(
            "Group {} cleaned up successfully: tenant={}, last_write_time={}s ago, expire_sec={}",
            group,
            tenant,
            now.saturating_sub(latest_ts),
            expire_sec
        );
    }

    Ok(())
}
