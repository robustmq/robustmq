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
use crate::core::notify::{
    send_notify_by_add_share_group_member, send_notify_by_delete_share_group,
    send_notify_by_delete_share_group_member, send_notify_by_set_share_group,
};
use crate::core::{cache::MetaCacheManager, group_leader::generate_group_leader};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::common::share_group::ShareGroupStorage;
use bytes::Bytes;
use common_base::tools::now_second;
use common_base::utils::serialize;
use common_base::uuid::unique_id;
use metadata_struct::mqtt::share_group::{ShareGroup, ShareGroupMember, ShareGroupParams};
use node_call::NodeCallManager;
use prost::Message as _;
use protocol::meta::meta_service_common::{
    AddShareGroupMemberReply, AddShareGroupMemberRequest, CreateShareGroupReply,
    CreateShareGroupRequest, DeleteShareGroupMemberReply, DeleteShareGroupMemberRequest,
    DeleteShareGroupReply, DeleteShareGroupRequest, ListShareGroupMemberReply,
    ListShareGroupMemberRequest, ListShareGroupReply, ListShareGroupRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn list_share_group_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListShareGroupRequest,
) -> Result<ListShareGroupReply, MetaServiceError> {
    let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());

    let leaders: Vec<ShareGroup> = if !req.tenant.is_empty() && !req.group.is_empty() {
        storage.get(&req.tenant, &req.group)?.into_iter().collect()
    } else if !req.tenant.is_empty() {
        storage.list_by_tenant(&req.tenant)?.into_values().collect()
    } else {
        storage.list_all()?.into_values().collect()
    };

    let groups = leaders
        .iter()
        .map(|l| l.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListShareGroupReply { groups })
}

pub fn list_share_group_member_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListShareGroupMemberRequest,
) -> Result<ListShareGroupMemberReply, MetaServiceError> {
    let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
    let members = if req.connect_id != 0 {
        storage.list_members(0, req.connect_id)?
    } else {
        storage.list_all_members()?
    };
    let members = members
        .iter()
        .map(|m| m.encode())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ListShareGroupMemberReply { members })
}

pub async fn create_share_group_by_req(
    cache_manager: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    req: &CreateShareGroupRequest,
) -> Result<CreateShareGroupReply, MetaServiceError> {
    let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&req.tenant, &req.group)?.is_some() {
        return Ok(CreateShareGroupReply {});
    }

    let target_broker_id =
        generate_group_leader(cache_manager, rocksdb_engine_handler, &req.tenant).await?;

    let sub_params = if req.params.is_empty() {
        ShareGroupParams::default()
    } else {
        ShareGroupParams::decode(&req.params)?
    };

    let leader = ShareGroup {
        uuid: unique_id(),
        tenant: req.tenant.clone(),
        group_name: req.group.clone(),
        sub_params,
        leader_broker: target_broker_id,
        create_time: now_second(),
    };

    let data = StorageData::new(
        StorageDataType::MqttSetGroupLeader,
        Bytes::copy_from_slice(&leader.encode()?),
    );
    raft_manager.write_data(&req.group, data).await?;
    send_notify_by_set_share_group(call_manager, leader).await?;

    Ok(CreateShareGroupReply {})
}

pub async fn delete_share_group_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    req: &DeleteShareGroupRequest,
) -> Result<DeleteShareGroupReply, MetaServiceError> {
    let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
    let Some(leader) = storage.get(&req.tenant, &req.group)? else {
        return Ok(DeleteShareGroupReply {});
    };

    let data = StorageData::new(
        StorageDataType::MqttDeleteGroupLeader,
        Bytes::copy_from_slice(&leader.encode()?),
    );
    raft_manager.write_data(&req.group, data).await?;
    send_notify_by_delete_share_group(call_manager, &req.tenant, &req.group).await?;

    Ok(DeleteShareGroupReply {})
}

pub async fn add_share_group_member_by_req(
    cache_manager: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    req: &AddShareGroupMemberRequest,
) -> Result<AddShareGroupMemberReply, MetaServiceError> {
    let member: ShareGroupMember = serialize::deserialize(&req.data)?;
    let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&member.tenant, &member.group_name)?.is_none() {
        let create_req = CreateShareGroupRequest {
            tenant: member.tenant.clone(),
            group: member.group_name.clone(),
            params: member.params.encode()?,
        };
        create_share_group_by_req(
            cache_manager,
            raft_manager,
            rocksdb_engine_handler,
            call_manager,
            &create_req,
        )
        .await?;
    }

    let data = StorageData::new(
        StorageDataType::MqttAddGroupMember,
        Bytes::copy_from_slice(&req.encode_to_vec()),
    );
    raft_manager.write_data(&member.group_name, data).await?;
    send_notify_by_add_share_group_member(call_manager, &member).await?;
    Ok(AddShareGroupMemberReply {})
}

pub async fn delete_share_group_member_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    req: &DeleteShareGroupMemberRequest,
) -> Result<DeleteShareGroupMemberReply, MetaServiceError> {
    let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
    if let Some(member) = storage.get_member(req.broker_id, req.connect_id, &req.sid)? {
        let data = StorageData::new(
            StorageDataType::MqttDeleteGroupMember,
            Bytes::copy_from_slice(&req.encode_to_vec()),
        );
        raft_manager.write_data(&member.group_name, data).await?;
        send_notify_by_delete_share_group_member(call_manager, &member).await?;
    };

    Ok(DeleteShareGroupMemberReply {})
}
