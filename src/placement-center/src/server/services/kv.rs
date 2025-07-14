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

use crate::core::error::PlacementCenterError;
use crate::raft::route::apply::RaftMachineApply;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::placement::kv::KvStorage;
use crate::storage::rocksdb::RocksDBEngine;
use prost::Message;
use protocol::placement_center::placement_center_kv::{
    DeleteReply, DeleteRequest, ExistsReply, ExistsRequest, GetPrefixReply, GetPrefixRequest,
    GetReply, GetRequest, ListShardReply, ListShardRequest, SetReply, SetRequest,
};
use std::sync::Arc;

pub async fn set_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &SetRequest,
) -> Result<SetReply, PlacementCenterError> {
    if req.key.is_empty() || req.value.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "key or value".to_string(),
        ));
    }

    let data = StorageData::new(StorageDataType::KvSet, SetRequest::encode_to_vec(req));
    raft_machine_apply.client_write(data).await?;

    Ok(SetReply::default())
}

pub async fn get_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetRequest,
) -> Result<GetReply, PlacementCenterError> {
    if req.key.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "key".to_string(),
        ));
    }

    let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
    let mut reply = GetReply::default();

    match kv_storage.get(req.key.clone()) {
        Ok(Some(data)) => {
            reply.value = data;
        }
        Ok(None) => {}
        Err(e) => return Err(PlacementCenterError::CommonError(e.to_string())),
    }

    Ok(reply)
}

pub async fn delete_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &DeleteRequest,
) -> Result<DeleteReply, PlacementCenterError> {
    if req.key.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "key".to_string(),
        ));
    }

    // Raft状态机用于存储节点数据
    let data = StorageData::new(StorageDataType::KvDelete, DeleteRequest::encode_to_vec(req));
    raft_machine_apply.client_write(data).await?;

    Ok(DeleteReply::default())
}

pub async fn exists_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ExistsRequest,
) -> Result<ExistsReply, PlacementCenterError> {
    if req.key.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "key".to_string(),
        ));
    }

    let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
    let flag = kv_storage.exists(req.key.clone())?;

    Ok(ExistsReply { flag })
}

pub async fn list_shard_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListShardRequest,
) -> Result<ListShardReply, PlacementCenterError> {
    if req.namespace.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "namespace".to_string(),
        ));
    }

    let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
    let shards_info = kv_storage.get_prefix(format!("/shard/{}/", req.namespace))?;

    Ok(ListShardReply { shards_info })
}

pub async fn get_prefix_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetPrefixRequest,
) -> Result<GetPrefixReply, PlacementCenterError> {
    if req.prefix.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "prefix".to_string(),
        ));
    }

    let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
    let values = kv_storage.get_prefix(req.prefix.clone())?;

    Ok(GetPrefixReply { values })
}
