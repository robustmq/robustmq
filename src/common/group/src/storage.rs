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

use crate::manager::OffsetManager;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_config::broker::broker_config;
use grpc_clients::meta::common::call::save_offset_data;
use protocol::meta::meta_service_common::{
    SaveOffsetData, SaveOffsetDataRequest, SaveOffsetDataRequestOffset,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, warn};

const OFFSET_SYNC_INTERVAL_MS: u64 = 20;

pub async fn start_offset_sync_task(
    manager: Arc<OffsetManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        sync_offsets(&manager).await;
        Ok(())
    };
    loop_select_ticket(ac_fn, OFFSET_SYNC_INTERVAL_MS, &stop_send).await;
}

async fn sync_offsets(manager: &OffsetManager) {
    // Drain update_group_info — collect dirty groups and clear the map.
    let dirty: Vec<(String, String)> = manager
        .update_group_info
        .iter()
        .map(|e| (e.tenant.clone(), e.group_name.clone()))
        .collect();

    if dirty.is_empty() {
        return;
    }
    manager.update_group_info.clear();

    // Build SaveOffsetData entries from offset_info.
    let mut offsets = Vec::with_capacity(dirty.len());
    for (tenant, group_name) in dirty {
        let key = manager.key(&tenant, &group_name);
        let Some(offset_map) = manager.offset_info.get(&key) else {
            warn!(
                "offset_sync_task: no offset_info for group '{}' tenant '{}'",
                group_name, tenant
            );
            continue;
        };
        let shard_offsets: Vec<SaveOffsetDataRequestOffset> = offset_map
            .iter()
            .map(|(shard_name, &offset)| SaveOffsetDataRequestOffset {
                shard_name: shard_name.clone(),
                offset,
            })
            .collect();

        offsets.push(SaveOffsetData {
            group: group_name,
            tenant,
            offsets: shard_offsets,
        });
    }

    if offsets.is_empty() {
        return;
    }

    let request = SaveOffsetDataRequest { offsets };
    let addrs = broker_config().get_meta_service_addr();
    if let Err(e) = save_offset_data(&manager.client_pool, &addrs, request).await {
        error!("offset_sync_task: failed to save offset data: {}", e);
    }
}
