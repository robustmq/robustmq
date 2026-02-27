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
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use grpc_clients::pool::ClientPool;
use node_call::NodeCallManager;
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn start_engine_delete_gc_thread(
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
    node_call_manager: Arc<NodeCallManager>,
    _client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        if let Err(e) = gc_shard(&raft_manager, &cache_manager, &node_call_manager).await {
            return Err(CommonError::CommonError(e.to_string()));
        };

        if let Err(e) = gc_segment(&raft_manager, &node_call_manager, &cache_manager).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, 10000, &stop_send).await;
}

async fn gc_shard(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    node_call_manager: &Arc<NodeCallManager>,
) -> Result<(), MetaServiceError> {
    for shard_name in cache_manager.get_wait_delete_shard_list() {
        let shard = if let Some(shard) = cache_manager.shard_list.get(&shard_name) {
            shard.clone()
        } else {
            continue;
        };

        send_notify_by_delete_shard(node_call_manager, shard).await?;
        delete_shard_by_real(cache_manager, raft_manager, &shard_name).await?;
    }
    Ok(())
}

async fn gc_segment(
    raft_manager: &Arc<MultiRaftManager>,
    node_call_manager: &Arc<NodeCallManager>,
    cache_manager: &Arc<MetaCacheManager>,
) -> Result<(), MetaServiceError> {
    for segment in cache_manager.get_wait_delete_segment_list() {
        if cache_manager
            .get_segment(&segment.shard_name, segment.segment_seq)
            .is_none()
        {
            cache_manager.remove_wait_delete_segment(&segment);
            continue;
        };

        send_notify_by_delete_segment(node_call_manager, segment.clone()).await?;
        delete_segment_by_real(cache_manager, raft_manager, &segment).await?;
    }
    Ok(())
}
