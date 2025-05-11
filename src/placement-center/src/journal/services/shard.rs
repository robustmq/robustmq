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

use super::segment::{
    build_segment, sync_save_segment_info, sync_save_segment_metadata_info, update_segment_status,
};
use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::journal::cache::JournalCacheManager;
use crate::journal::controller::call_node::{
    update_cache_by_set_segment, update_cache_by_set_segment_meta, update_cache_by_set_shard,
    JournalInnerCallManager,
};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::journal::shard::ShardStorage;
use common_base::tools::{now_mills, unique_id};
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::segment::SegmentStatus;
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::{JournalShard, JournalShardConfig, JournalShardStatus};
use protocol::placement_center::placement_center_journal::{
    CreateShardReply, CreateShardRequest, DeleteShardReply, DeleteShardRequest, ListShardReply,
    ListShardRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

pub async fn list_shard_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListShardRequest,
) -> Result<ListShardReply, PlacementCenterError> {
    if req.cluster_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            req.cluster_name.clone(),
        ));
    }

    let shard_storage = ShardStorage::new(rocksdb_engine_handler.clone());
    let binary_shards = if req.namespace.is_empty() && req.shard_name.is_empty() {
        shard_storage.list_by_cluster(&req.cluster_name)?
    } else if !req.namespace.is_empty() && req.shard_name.is_empty() {
        shard_storage.list_by_cluster_namespace(&req.cluster_name, &req.namespace)?
    } else {
        match shard_storage.get(&req.cluster_name, &req.namespace, &req.shard_name)? {
            Some(shard) => vec![shard],
            None => Vec::new(),
        }
    };

    let shards: Vec<JournalShard> = binary_shards.into_iter().collect();

    let shards_data = serde_json::to_vec(&shards)?;

    Ok(ListShardReply {
        shards: shards_data,
    })
}

pub async fn create_shard_by_req(
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateShardRequest,
) -> Result<CreateShardReply, PlacementCenterError> {
    if cluster_cache.get_cluster(&req.cluster_name).is_none() {
        return Err(PlacementCenterError::ClusterDoesNotExist(
            req.cluster_name.clone(),
        ));
    }

    // Check that the number of available nodes in the cluster is sufficient
    let num = cluster_cache.get_broker_num(&req.cluster_name) as u32;
    let shard_config: JournalShardConfig =
        serde_json::from_slice::<JournalShardConfig>(&req.shard_config)?;
    if num < shard_config.replica_num {
        return Err(PlacementCenterError::NotEnoughNodes(
            shard_config.replica_num,
            num,
        ));
    }

    let shard = if let Some(shard) =
        engine_cache.get_shard(&req.cluster_name, &req.namespace, &req.shard_name)
    {
        shard
    } else {
        let shard = JournalShard {
            shard_uid: unique_id(),
            cluster_name: req.cluster_name.clone(),
            namespace: req.namespace.clone(),
            shard_name: req.shard_name.clone(),
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            status: JournalShardStatus::Run,
            config: shard_config,
            create_time: now_mills(),
        };

        sync_save_shard_info(raft_machine_apply, &shard).await?;

        shard
    };

    let mut segment = if let Some(segment) = engine_cache.get_segment(
        &shard.cluster_name,
        &shard.namespace,
        &shard.shard_name,
        shard.active_segment_seq,
    ) {
        segment
    } else {
        let segment = build_segment(&shard, engine_cache, cluster_cache, 0).await?;

        sync_save_segment_info(raft_machine_apply, &segment).await?;

        let metadata = JournalSegmentMetadata {
            cluster_name: segment.cluster_name.clone(),
            namespace: segment.namespace.clone(),
            shard_name: segment.shard_name.clone(),
            segment_seq: segment.segment_seq,
            start_offset: 0,
            end_offset: -1,
            start_timestamp: 0,
            end_timestamp: -1,
        };

        sync_save_segment_metadata_info(raft_machine_apply, &metadata).await?;
        update_cache_by_set_segment_meta(&req.cluster_name, call_manager, client_pool, metadata)
            .await?;

        segment
    };

    update_segment_status(
        engine_cache,
        raft_machine_apply,
        &segment,
        SegmentStatus::Write,
    )
    .await?;
    segment.status = SegmentStatus::Write;

    // update segment cache
    update_cache_by_set_shard(&req.cluster_name, call_manager, client_pool, shard.clone()).await?;

    update_cache_by_set_segment(
        &segment.cluster_name,
        call_manager,
        client_pool,
        segment.clone(),
    )
    .await?;

    let replica: Vec<u64> = segment.replicas.iter().map(|rep| rep.node_id).collect();
    Ok(CreateShardReply {
        segment_no: segment.segment_seq,
        replica,
    })
}

pub async fn delete_shard_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    engine_cache: &Arc<JournalCacheManager>,
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteShardRequest,
) -> Result<DeleteShardReply, PlacementCenterError> {
    let mut shard = if let Some(shard) =
        engine_cache.get_shard(&req.cluster_name, &req.namespace, &req.shard_name)
    {
        shard
    } else {
        return Err(PlacementCenterError::ShardDoesNotExist(
            req.cluster_name.clone(),
        ));
    };

    update_shard_status(
        raft_machine_apply,
        engine_cache,
        &shard,
        JournalShardStatus::PrepareDelete,
    )
    .await?;

    shard.status = JournalShardStatus::PrepareDelete;
    engine_cache.add_wait_delete_shard(&shard);

    update_cache_by_set_shard(&req.cluster_name, call_manager, client_pool, shard.clone()).await?;

    Ok(DeleteShardReply::default())
}

pub async fn update_start_segment_by_shard(
    raft_machine_apply: &Arc<RaftMachineApply>,
    engine_cache: &Arc<JournalCacheManager>,
    shard: &mut JournalShard,
    segment_no: u32,
) -> Result<(), PlacementCenterError> {
    shard.start_segment_seq = segment_no;
    sync_save_shard_info(raft_machine_apply, shard).await?;
    engine_cache.set_shard(shard);
    Ok(())
}

pub async fn update_last_segment_by_shard(
    raft_machine_apply: &Arc<RaftMachineApply>,
    engine_cache: &Arc<JournalCacheManager>,
    shard: &mut JournalShard,
    segment_no: u32,
) -> Result<(), PlacementCenterError> {
    shard.last_segment_seq = segment_no;
    sync_save_shard_info(raft_machine_apply, shard).await?;
    engine_cache.set_shard(shard);
    Ok(())
}

async fn sync_save_shard_info(
    raft_machine_apply: &Arc<RaftMachineApply>,
    shard: &JournalShard,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::JournalSetShard,
        serde_json::to_vec(&shard)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_shard_info(
    raft_machine_apply: &Arc<RaftMachineApply>,
    shard: &JournalShard,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::JournalDeleteShard,
        serde_json::to_vec(&shard)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}

pub async fn update_shard_status(
    raft_machine_apply: &Arc<RaftMachineApply>,
    engine_cache: &Arc<JournalCacheManager>,
    shard: &JournalShard,
    status: JournalShardStatus,
) -> Result<(), PlacementCenterError> {
    let mut new_shard = shard.clone();
    new_shard.status = status;
    sync_save_shard_info(raft_machine_apply, &new_shard).await?;
    engine_cache.set_shard(&new_shard);
    Ok(())
}

#[cfg(test)]
mod tests {}
