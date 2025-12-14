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
use crate::storage::mqtt::blacklist::MqttBlackListStorage;
use crate::{
    raft::route::data::{StorageData, StorageDataType},
    storage::mqtt::acl::AclStorage,
};
use common_base::utils::serialize::encode_to_bytes;
use protocol::meta::meta_service_mqtt::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest, DeleteAclReply,
    DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest, ListAclReply, ListAclRequest,
    ListBlacklistReply, ListBlacklistRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

// ACL Operations
pub fn list_acl_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    _req: &ListAclRequest,
) -> Result<ListAclReply, MetaServiceError> {
    let acl_storage = AclStorage::new(rocksdb_engine_handler.clone());
    let acls = acl_storage
        .list_all()?
        .into_iter()
        .map(|acl| acl.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListAclReply { acls })
}

pub async fn create_acl_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &CreateAclRequest,
) -> Result<CreateAclReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::MqttSetAcl, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    Ok(CreateAclReply {})
}

pub async fn delete_acl_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &DeleteAclRequest,
) -> Result<DeleteAclReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::MqttDeleteAcl, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    Ok(DeleteAclReply {})
}

// Blacklist Operations
pub fn list_blacklist_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    _req: &ListBlacklistRequest,
) -> Result<ListBlacklistReply, MetaServiceError> {
    let blacklist_storage = MqttBlackListStorage::new(rocksdb_engine_handler.clone());
    let blacklists = blacklist_storage
        .list_all()?
        .into_iter()
        .map(|item| item.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListBlacklistReply { blacklists })
}

pub async fn create_blacklist_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &CreateBlacklistRequest,
) -> Result<CreateBlacklistReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::MqttSetBlacklist, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    Ok(CreateBlacklistReply {})
}

pub async fn delete_blacklist_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &DeleteBlacklistRequest,
) -> Result<DeleteBlacklistReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::MqttDeleteBlacklist, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    Ok(DeleteBlacklistReply {})
}
