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

use crate::controller::call_broker::call::BrokerCallManager;
use crate::controller::call_broker::storage::{
    update_cache_by_delete_segment, update_cache_by_delete_shard,
};
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::core::segment::delete_segment_by_real;
use crate::core::shard::{delete_shard_by_real, update_shard_status};
use crate::raft::manager::MultiRaftManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::SegmentStatus;
use metadata_struct::storage::shard::EngineShardStatus;
use std::sync::Arc;
use tracing::warn;

pub async fn gc_shard_thread(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), MetaServiceError> {
    for shard_name in cache_manager.get_wait_delete_shard_list() {
        let shard = if let Some(shard) = cache_manager.shard_list.get(&shard_name) {
            shard.clone()
        } else {
            continue;
        };

        if shard.status != EngineShardStatus::PrepareDelete {
            warn!(
                "shard {} in wait_delete_shard_list is in the wrong state, current state is {:?}",
                shard.shard_name, shard.status
            );
            cache_manager.remove_wait_delete_shard(&shard_name);
            continue;
        }

        update_shard_status(
            raft_manager,
            cache_manager,
            call_manager,
            client_pool,
            &shard_name,
            EngineShardStatus::Deleting,
        )
        .await?;

        update_cache_by_delete_shard(call_manager, client_pool, shard).await?;
        delete_shard_by_real(cache_manager, raft_manager, &shard_name).await?;
    }
    Ok(())
}

pub async fn gc_segment_thread(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), MetaServiceError> {
    for segment in cache_manager.get_wait_delete_segment_list() {
        if cache_manager
            .get_segment(&segment.shard_name, segment.segment_seq)
            .is_none()
        {
            cache_manager.remove_wait_delete_segment(&segment);
            continue;
        };

        if segment.status != SegmentStatus::PreDelete {
            warn!(
                "segment {} in wait_delete_segment_list is in the wrong state, current state is {:?}",
                segment.name(),
                segment.status
            );
            cache_manager.remove_wait_delete_segment(&segment);
            continue;
        }

        update_cache_by_delete_segment(call_manager, client_pool, segment.clone()).await?;
        delete_segment_by_real(cache_manager, raft_manager, &segment).await?;
    }
    Ok(())
}
