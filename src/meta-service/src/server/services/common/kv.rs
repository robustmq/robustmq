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

use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::common::kv::KvStorage;
use common_base::utils::serialize::encode_to_bytes;
use protocol::meta::meta_service_common::{
    DeleteReply, DeleteRequest, ExistsReply, ExistsRequest, GetPrefixReply, GetPrefixRequest,
    GetReply, GetRequest, SetReply, SetRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

// Helper: Validate non-empty field
fn validate_non_empty(value: &str, field_name: &str) -> Result<(), MetaServiceError> {
    if value.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            field_name.to_string(),
        ));
    }
    Ok(())
}

// KV Operations
pub async fn set_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &SetRequest,
) -> Result<SetReply, MetaServiceError> {
    validate_non_empty(&req.key, "key")?;
    validate_non_empty(&req.value, "value")?;

    let data = StorageData::new(StorageDataType::KvSet, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    Ok(SetReply::default())
}

pub async fn get_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetRequest,
) -> Result<GetReply, MetaServiceError> {
    validate_non_empty(&req.key, "key")?;

    let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
    let value = kv_storage
        .get(req.key.clone())
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))?
        .unwrap_or_default();

    Ok(GetReply { value })
}

pub async fn delete_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &DeleteRequest,
) -> Result<DeleteReply, MetaServiceError> {
    validate_non_empty(&req.key, "key")?;

    let data = StorageData::new(StorageDataType::KvDelete, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    Ok(DeleteReply::default())
}

pub async fn exists_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ExistsRequest,
) -> Result<ExistsReply, MetaServiceError> {
    validate_non_empty(&req.key, "key")?;

    let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
    let flag = kv_storage.exists(req.key.clone())?;

    Ok(ExistsReply { flag })
}

pub async fn get_prefix_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetPrefixRequest,
) -> Result<GetPrefixReply, MetaServiceError> {
    validate_non_empty(&req.prefix, "prefix")?;

    let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
    let values = kv_storage.get_prefix(req.prefix.clone())?;

    Ok(GetPrefixReply { values })
}
