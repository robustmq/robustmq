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
use protocol::placement_center::placement_center_mqtt::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest, DeleteAclReply,
    DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest, ListAclReply, ListAclRequest,
    ListBlacklistReply, ListBlacklistRequest,
};
use rocksdb_engine::RocksDBEngine;
use tonic::{Request, Response, Status};

use crate::storage::mqtt::blacklist::MqttBlackListStorage;
use crate::{
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
    storage::mqtt::acl::AclStorage,
};

pub fn list_acl_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListAclRequest>,
) -> Result<Response<ListAclReply>, Status> {
    let req = request.into_inner();
    let acl_storage = AclStorage::new(rocksdb_engine_handler.clone());
    let list = acl_storage.list(&req.cluster_name)?;
    let mut acls = Vec::new();
    for acl in list {
        acls.push(acl.encode()?);
    }
    Ok(Response::new(ListAclReply { acls }))
}
pub async fn create_acl_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<CreateAclRequest>,
) -> Result<Response<CreateAclReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttSetAcl,
        CreateAclRequest::encode_to_vec(&req),
    );

    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };
    Ok(Response::new(CreateAclReply::default()))
}

pub async fn delete_acl_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<DeleteAclRequest>,
) -> Result<Response<DeleteAclReply>, Status> {
    let req = request.into_inner();

    let data = StorageData::new(
        StorageDataType::MqttDeleteAcl,
        DeleteAclRequest::encode_to_vec(&req),
    );

    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };
    Ok(Response::new(DeleteAclReply::default()))
}

pub fn list_blacklist_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListBlacklistRequest>,
) -> Result<Response<ListBlacklistReply>, Status> {
    let req = request.into_inner();
    let blacklist_storage = MqttBlackListStorage::new(rocksdb_engine_handler.clone());
    match blacklist_storage.list(&req.cluster_name) {
        Ok(list) => {
            let mut blacklists = Vec::new();
            for acl in list {
                match acl.encode() {
                    Ok(data) => {
                        blacklists.push(data);
                    }
                    Err(e) => {
                        return Err(Status::cancelled(e.to_string()));
                    }
                }
            }
            Ok(Response::new(ListBlacklistReply { blacklists }))
        }
        Err(e) => {
            println!("error:{:?}", e);
            Err(Status::internal(e.to_string()))
        }
    }
}

pub async fn delete_blacklist_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<DeleteBlacklistRequest>,
) -> Result<Response<DeleteBlacklistReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttDeleteBlacklist,
        DeleteBlacklistRequest::encode_to_vec(&req),
    );

    match raft_machine_apply.client_write(data).await {
        Ok(_) => Ok(Response::new(DeleteBlacklistReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn create_blacklist_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<CreateBlacklistRequest>,
) -> Result<Response<CreateBlacklistReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttSetBlacklist,
        CreateBlacklistRequest::encode_to_vec(&req),
    );

    match raft_machine_apply.client_write(data).await {
        Ok(_) => Ok(Response::new(CreateBlacklistReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}
