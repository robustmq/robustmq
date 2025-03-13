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
    CreateAclRequest, DeleteAclRequest, ListAclRequest,
};
use rocksdb_engine::RocksDBEngine;

use crate::{
    core::error::PlacementCenterError,
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
    storage::mqtt::acl::AclStorage,
};

pub fn list_acl_by_req(
    req: &ListAclRequest,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
    let acl_storage = AclStorage::new(rocksdb_engine_handler.clone());
    let list = acl_storage.list(&req.cluster_name)?;
    let mut acls = Vec::new();
    for acl in list {
        acls.push(acl.encode()?);
    }
    Ok(acls)
}

pub async fn create_acl_by_req(
    req: &CreateAclRequest,
    raft_machine_apply: &Arc<RaftMachineApply>,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttSetAcl,
        CreateAclRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(())
}

pub async fn delete_acl_by_req(
    req: &DeleteAclRequest,
    raft_machine_apply: &Arc<RaftMachineApply>,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteAcl,
        DeleteAclRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(())
}
