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

use crate::core::notify::send_notify_by_delete_group;
use crate::storage::common::offset::OffsetStorage;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

// Scan every 5 minutes
const GROUP_GC_INTERVAL_MS: u64 = 5 * 60 * 1000;

pub async fn start_group_gc_thread(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    node_call_manager: Arc<NodeCallManager>,
    group_offset_expire_sec: u64,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        if let Err(e) = gc_expired_groups(
            &rocksdb_engine_handler,
            &node_call_manager,
            group_offset_expire_sec,
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

        // Delete all shard offset records for this group from RocksDB.
        let group_offsets = storage.group_offset(&tenant, &group)?;
        for offset in &group_offsets {
            if let Err(e) = storage.delete(&offset.tenant, &offset.group, &offset.shard_name) {
                warn!(
                    "Failed to delete offset record: tenant={}, group={}, shard={}, error={}",
                    offset.tenant, offset.group, offset.shard_name, e
                );
            }
        }

        // Notify all broker nodes to clean up their in-memory group state.
        if let Err(e) = send_notify_by_delete_group(node_call_manager, &tenant, &group).await {
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
