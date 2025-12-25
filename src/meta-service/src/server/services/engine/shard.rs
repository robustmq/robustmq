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
    update_cache_by_set_segment, update_cache_by_set_segment_meta, update_cache_by_set_shard,
};
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::core::segment::{
    create_segment, sync_save_segment_info, sync_save_segment_metadata_info, update_segment_status,
};
use crate::core::shard::{sync_save_shard_info, update_shard_status};
use crate::raft::manager::MultiRaftManager;
use crate::storage::journal::shard::ShardStorage;
use common_base::tools::{now_millis, unique_id};
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::SegmentStatus;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::{
    EngineShard, EngineShardConfig, EngineShardStatus, EngineType,
};
use protocol::meta::meta_service_journal::{
    CreateShardReply, CreateShardRequest, DeleteShardReply, DeleteShardRequest, ListShardReply,
    ListShardRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn list_shard_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListShardRequest,
) -> Result<ListShardReply, MetaServiceError> {
    let shard_storage = ShardStorage::new(rocksdb_engine_handler.clone());
    let binary_shards = if req.shard_name.is_empty() {
        shard_storage.all_shard()?
    } else {
        match shard_storage.get(&req.shard_name)? {
            Some(shard) => vec![shard],
            None => Vec::new(),
        }
    };

    let shards: Vec<EngineShard> = binary_shards.into_iter().collect();

    let shards_data = shards
        .into_iter()
        .map(|shard| shard.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListShardReply {
        shards: shards_data,
    })
}

pub async fn create_shard_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateShardRequest,
) -> Result<CreateShardReply, MetaServiceError> {
    // Check that the number of available nodes is sufficient
    let num = cache_manager.node_list.len() as u32;

    let shard_config: EngineShardConfig = EngineShardConfig::decode(&req.shard_config)?;
    if num < shard_config.replica_num {
        return Err(MetaServiceError::NotEnoughEngineNodes(
            shard_config.replica_num,
            num,
        ));
    }

    let shard = if let Some(shard) = cache_manager.get_shard(&req.shard_name) {
        shard
    } else {
        let shard = EngineShard {
            shard_uid: unique_id(),
            shard_name: req.shard_name.clone(),
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            status: EngineShardStatus::Run,
            config: shard_config.clone(),
            replica_num: shard_config.replica_num,
            engine_type: EngineType::Segment,
            create_time: now_millis(),
        };

        sync_save_shard_info(raft_manager, &shard).await?;

        shard
    };

    let mut segment = if let Some(segment) =
        cache_manager.get_segment(&shard.shard_name, shard.active_segment_seq)
    {
        segment
    } else {
        let segment = create_segment(&shard, cache_manager, 0).await?;

        sync_save_segment_info(raft_manager, &segment).await?;

        let metadata = EngineSegmentMetadata {
            shard_name: segment.shard_name.clone(),
            segment_seq: segment.segment_seq,
            start_offset: 0,
            end_offset: -1,
            start_timestamp: 0,
            end_timestamp: -1,
        };

        sync_save_segment_metadata_info(raft_manager, &metadata).await?;
        update_cache_by_set_segment_meta(call_manager, client_pool, metadata).await?;
        segment
    };

    update_segment_status(cache_manager, raft_manager, &segment, SegmentStatus::Write).await?;
    segment.status = SegmentStatus::Write;

    update_cache_by_set_shard(call_manager, client_pool, shard.clone()).await?;
    update_cache_by_set_segment(call_manager, client_pool, segment.clone()).await?;

    let replica: Vec<u64> = segment.replicas.iter().map(|rep| rep.node_id).collect();
    Ok(CreateShardReply {
        segment_no: segment.segment_seq,
        replica,
    })
}

pub async fn delete_shard_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteShardRequest,
) -> Result<DeleteShardReply, MetaServiceError> {
    let mut shard = if let Some(shard) = cache_manager.get_shard(&req.shard_name) {
        shard
    } else {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    update_shard_status(
        raft_manager,
        cache_manager,
        &shard,
        EngineShardStatus::PrepareDelete,
    )
    .await?;

    shard.status = EngineShardStatus::PrepareDelete;
    cache_manager.add_wait_delete_shard(&shard);

    update_cache_by_set_shard(call_manager, client_pool, shard.clone()).await?;

    Ok(DeleteShardReply::default())
}

#[cfg(test)]
mod tests {}
