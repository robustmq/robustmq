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

use std::sync::Arc;

use common_base::tools::{now_mills, unique_id};
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::shard::JournalShard;
use protocol::placement_center::placement_center_journal::{CreateShardReply, CreateShardRequest};
use rocksdb_engine::RocksDBEngine;

use super::segmet::create_first_segment;
use crate::cache::journal::JournalCacheManager;
use crate::cache::placement::PlacementCacheManager;
use crate::controller::journal::call_node::{
    update_cache_by_add_segment, update_cache_by_add_shard, JournalInnerCallManager,
};
use crate::core::error::PlacementCenterError;
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::journal::segment::is_seal_up_segment;

pub async fn create_shard_by_req(
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<JournalInnerCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    client_pool: &Arc<ClientPool>,
    req: &CreateShardRequest,
) -> Result<CreateShardReply, PlacementCenterError> {
    // Check that the number of available nodes in the cluster is sufficient
    let num = cluster_cache.get_broker_num(&req.cluster_name) as u32;

    if num < req.replica {
        return Err(PlacementCenterError::NotEnoughNodes(req.replica, num));
    }

    let shard = if let Some(shard) =
        engine_cache.get_shard(&req.cluster_name, &req.namespace, &req.shard_name)
    {
        shard
    } else {
        let shard_info = JournalShard {
            shard_uid: unique_id(),
            cluster_name: req.cluster_name.clone(),
            namespace: req.namespace.clone(),
            shard_name: req.shard_name.clone(),
            replica: req.replica,
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            create_time: now_mills(),
        };

        let data = StorageData::new(
            StorageDataType::JournalCreateShard,
            serde_json::to_vec(&shard_info)?,
        );

        if let Some(_) = raft_machine_apply.client_write(data).await? {
            update_cache_by_add_shard(&req.cluster_name, call_manager, client_pool, shard_info)
                .await?;
        } else {
            return Err(PlacementCenterError::ExecutionResultIsEmpty);
        }
        shard_info
    };

    if is_create_next_segment(engine_cache, &shard) {}
    {
        // Create first Segment
        let segment =
            create_first_segment(&shard, engine_cache, cluster_cache, rocksdb_engine_handler)?;

        // Update storage engine node cache by segment
        update_cache_by_add_segment(
            &req.cluster_name,
            call_manager,
            client_pool,
            segment.clone(),
        )
        .await?;
    }

    // Handle situations where the Shard already exists in order to support interface idempotency
    // Create first Segment
    let segment = create_first_segment(
        &shard_info,
        engine_cache,
        cluster_cache,
        rocksdb_engine_handler,
    )?;

    // Update storage engine node cache by segment
    update_cache_by_add_segment(
        &req.cluster_name,
        call_manager,
        client_pool,
        segment.clone(),
    )
    .await?;

    let replica: Vec<u64> = segment.replicas.iter().map(|rep| rep.node_id).collect();
    return Ok(CreateShardReply {
        segment_no: segment.segment_seq,
        replica,
    });

    let shard = shard_res.unwrap();
    if let Some(segment) = engine_cache.get_segment(
        &req.cluster_name,
        &req.namespace,
        &req.shard_name,
        shard.active_segment_seq,
    ) {
        if !is_seal_up_segment(segment.status) {
            let replicas = segment.replicas.iter().map(|rep| rep.node_id).collect();
            return Ok(CreateShardReply {
                segment_no: shard.active_segment_seq,
                replica: replicas,
            });
        }
    }

    let replica: Vec<u64> = segment.replicas.iter().map(|rep| rep.node_id).collect();
    return Ok(CreateShardReply {
        segment_no: segment.segment_seq,
        replica,
    });
}

fn is_create_next_segment(
    engine_cache: &Arc<JournalCacheManager>,
    shard_info: &JournalShard,
) -> bool {
    if let Some(segment) = engine_cache.get_segment(
        &shard_info.cluster_name,
        &shard_info.namespace,
        &shard_info.shard_name,
        shard_info.active_segment_seq,
    ) {
        return is_seal_up_segment(segment.status);
    }
    true
}
