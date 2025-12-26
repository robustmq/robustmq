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
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::core::segment::create_segment;
use crate::core::shard::{create_shard, update_shard_status};
use crate::raft::manager::MultiRaftManager;
use crate::storage::journal::shard::ShardStorage;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::shard::{EngineShard, EngineShardConfig, EngineShardStatus};
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

    let shard: EngineShard = create_shard(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        &req.shard_name,
        shard_config,
    )
    .await?;

    let segment = create_segment(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        &shard,
        0,
        0,
    )
    .await?;

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
    if cache_manager.shard_list.contains_key(&req.shard_name) {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    update_shard_status(
        raft_manager,
        cache_manager,
        call_manager,
        client_pool,
        &req.shard_name,
        EngineShardStatus::PrepareDelete,
    )
    .await?;

    cache_manager.add_wait_delete_shard(&req.shard_name);

    Ok(DeleteShardReply::default())
}
