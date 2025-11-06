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

use prost::Message;
use protocol::meta::meta_service_mqtt::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest, DeleteAclReply,
    DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest, ListAclReply, ListAclRequest,
    ListBlacklistReply, ListBlacklistRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;

use crate::core::error::MetaServiceError;
use crate::storage::mqtt::blacklist::MqttBlackListStorage;
use crate::{
    raft::route::{
        apply::RaftMachineManager,
        data::{StorageData, StorageDataType},
    },
    storage::mqtt::acl::AclStorage,
};

pub fn list_acl_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListAclRequest,
) -> Result<ListAclReply, MetaServiceError> {
    let acl_storage = AclStorage::new(rocksdb_engine_handler.clone());
    let list = acl_storage.list(&req.cluster_name)?;
    let mut acls = Vec::new();
    for acl in list {
        acls.push(acl.encode()?);
    }
    Ok(ListAclReply { acls })
}

pub async fn create_acl_by_req(
    raft_machine_apply: &Arc<RaftMachineManager>,
    req: &CreateAclRequest,
) -> Result<CreateAclReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttSetAcl,
        CreateAclRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(CreateAclReply {})
}

pub async fn delete_acl_by_req(
    raft_machine_apply: &Arc<RaftMachineManager>,
    req: &DeleteAclRequest,
) -> Result<DeleteAclReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteAcl,
        DeleteAclRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(DeleteAclReply {})
}

pub fn list_blacklist_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListBlacklistRequest,
) -> Result<ListBlacklistReply, MetaServiceError> {
    let blacklist_storage = MqttBlackListStorage::new(rocksdb_engine_handler.clone());
    let list = blacklist_storage.list(&req.cluster_name)?;
    let mut blacklists = Vec::new();
    for acl in list {
        blacklists.push(acl.encode()?);
    }
    Ok(ListBlacklistReply { blacklists })
}

pub async fn delete_blacklist_by_req(
    raft_machine_apply: &Arc<RaftMachineManager>,
    req: &DeleteBlacklistRequest,
) -> Result<DeleteBlacklistReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteBlacklist,
        DeleteBlacklistRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(DeleteBlacklistReply {})
}

pub async fn create_blacklist_by_req(
    raft_machine_apply: &Arc<RaftMachineManager>,
    req: &CreateBlacklistRequest,
) -> Result<CreateBlacklistReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttSetBlacklist,
        CreateBlacklistRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(CreateBlacklistReply {})
}
