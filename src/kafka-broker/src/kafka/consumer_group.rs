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

use crate::core::coordinator::GroupCoordinator;
use crate::core::coordinator_locator::{is_coordinator_node, resolve_group_coordinator};
use crate::core::join::JoinGroupParams;
use crate::core::sync::SyncGroupParams;
use crate::handler::tenant::get_tenant;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::delete_groups_response::DeletableGroupResult;
use kafka_protocol::messages::describe_groups_response::{DescribedGroup, DescribedGroupMember};
use kafka_protocol::messages::find_coordinator_response::Coordinator;
use kafka_protocol::messages::join_group_response::JoinGroupResponseMember;
use kafka_protocol::messages::leave_group_response::MemberResponse;
use kafka_protocol::messages::list_groups_response::ListedGroup;
use kafka_protocol::messages::{
    DeleteGroupsRequest, DeleteGroupsResponse, DescribeGroupsRequest, DescribeGroupsResponse,
    FindCoordinatorRequest, FindCoordinatorResponse, GroupId, HeartbeatRequest, HeartbeatResponse,
    JoinGroupRequest, JoinGroupResponse, LeaveGroupRequest, LeaveGroupResponse, ListGroupsRequest,
    ListGroupsResponse, SyncGroupRequest, SyncGroupResponse,
};
use kafka_protocol::protocol::StrBytes;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

const KEY_TYPE_GROUP: i8 = 0;

pub async fn process_find_coordinator(
    sdm: &Arc<StorageDriverManager>,
    req: &FindCoordinatorRequest,
) -> Option<KafkaPacket> {
    let resolved = if req.key_type == KEY_TYPE_GROUP {
        resolve_group_coordinator(sdm).await
    } else {
        Err(ResponseError::CoordinatorNotAvailable.code())
    };

    let (error_code, node_id, host, port) = match resolved {
        Ok((node_id, host, port)) => (0, node_id, host, port),
        Err(code) => (code, -1, String::new(), -1),
    };

    let keys = if req.coordinator_keys.is_empty() {
        vec![req.key.clone()]
    } else {
        req.coordinator_keys.clone()
    };
    let coordinators = keys
        .into_iter()
        .map(|key| {
            Coordinator::default()
                .with_key(key)
                .with_node_id(node_id.into())
                .with_host(StrBytes::from(host.clone()))
                .with_port(port)
                .with_error_code(error_code)
        })
        .collect();

    Some(KafkaPacket::FindCoordinatorResponse(
        FindCoordinatorResponse::default()
            .with_error_code(error_code)
            .with_node_id(node_id.into())
            .with_host(StrBytes::from(host))
            .with_port(port)
            .with_coordinators(coordinators),
    ))
}

pub async fn process_join_group(
    coordinator: &GroupCoordinator,
    sdm: &Arc<StorageDriverManager>,
    api_version: i16,
    client_id: String,
    req: &JoinGroupRequest,
) -> Option<KafkaPacket> {
    if !is_coordinator_node(sdm).await {
        return Some(join_group_error(ResponseError::NotCoordinator.code()));
    }

    let params = JoinGroupParams {
        group_id: req.group_id.to_string(),
        member_id: req.member_id.to_string(),
        group_instance_id: req.group_instance_id.as_ref().map(|s| s.to_string()),
        client_id,
        session_timeout_ms: req.session_timeout_ms,
        rebalance_timeout_ms: req.rebalance_timeout_ms,
        protocol_type: req.protocol_type.to_string(),
        protocols: req
            .protocols
            .iter()
            .map(|p| (p.name.to_string(), p.metadata.clone()))
            .collect(),
        require_member_id: api_version >= 4,
    };

    let result = coordinator.join_group(params).await;

    let members = result
        .members
        .into_iter()
        .map(|(id, metadata)| {
            JoinGroupResponseMember::default()
                .with_member_id(StrBytes::from(id))
                .with_metadata(metadata)
        })
        .collect();

    Some(KafkaPacket::JoinGroupResponse(
        JoinGroupResponse::default()
            .with_error_code(result.error_code)
            .with_generation_id(result.generation_id)
            .with_protocol_type(result.protocol_type.map(StrBytes::from))
            .with_protocol_name(result.protocol_name.map(StrBytes::from))
            .with_leader(StrBytes::from(result.leader_id))
            .with_member_id(StrBytes::from(result.member_id))
            .with_members(members),
    ))
}

pub async fn process_heartbeat(
    coordinator: &GroupCoordinator,
    sdm: &Arc<StorageDriverManager>,
    req: &HeartbeatRequest,
) -> Option<KafkaPacket> {
    if !is_coordinator_node(sdm).await {
        return Some(KafkaPacket::HeartbeatResponse(
            HeartbeatResponse::default().with_error_code(ResponseError::NotCoordinator.code()),
        ));
    }

    let error_code = coordinator.heartbeat(
        &req.group_id.to_string(),
        &req.member_id.to_string(),
        req.generation_id,
    );

    Some(KafkaPacket::HeartbeatResponse(
        HeartbeatResponse::default().with_error_code(error_code),
    ))
}

pub async fn process_leave_group(
    coordinator: &GroupCoordinator,
    sdm: &Arc<StorageDriverManager>,
    req: &LeaveGroupRequest,
) -> Option<KafkaPacket> {
    if !is_coordinator_node(sdm).await {
        return Some(KafkaPacket::LeaveGroupResponse(
            LeaveGroupResponse::default().with_error_code(ResponseError::NotCoordinator.code()),
        ));
    }

    let single = req.members.is_empty();
    let member_ids: Vec<String> = if single {
        vec![req.member_id.to_string()]
    } else {
        req.members
            .iter()
            .map(|m| m.member_id.to_string())
            .collect()
    };

    let results = coordinator.leave_group(&req.group_id.to_string(), &member_ids);

    let top_error = if single {
        results.first().map(|(_, code)| *code).unwrap_or(0)
    } else {
        0
    };
    let members = results
        .into_iter()
        .map(|(id, code)| {
            MemberResponse::default()
                .with_member_id(StrBytes::from(id))
                .with_error_code(code)
        })
        .collect();

    Some(KafkaPacket::LeaveGroupResponse(
        LeaveGroupResponse::default()
            .with_error_code(top_error)
            .with_members(members),
    ))
}

pub async fn process_sync_group(
    coordinator: &GroupCoordinator,
    sdm: &Arc<StorageDriverManager>,
    req: &SyncGroupRequest,
) -> Option<KafkaPacket> {
    if !is_coordinator_node(sdm).await {
        return Some(KafkaPacket::SyncGroupResponse(
            SyncGroupResponse::default().with_error_code(ResponseError::NotCoordinator.code()),
        ));
    }

    let params = SyncGroupParams {
        group_id: req.group_id.to_string(),
        member_id: req.member_id.to_string(),
        generation_id: req.generation_id,
        assignments: req
            .assignments
            .iter()
            .map(|a| (a.member_id.to_string(), a.assignment.clone()))
            .collect(),
    };

    let result = coordinator.sync_group(params).await;

    Some(KafkaPacket::SyncGroupResponse(
        SyncGroupResponse::default()
            .with_error_code(result.error_code)
            .with_protocol_type(result.protocol_type.map(StrBytes::from))
            .with_protocol_name(result.protocol_name.map(StrBytes::from))
            .with_assignment(result.assignment),
    ))
}

pub async fn process_describe_groups(
    coordinator: &GroupCoordinator,
    sdm: &Arc<StorageDriverManager>,
    req: &DescribeGroupsRequest,
) -> Option<KafkaPacket> {
    let authorized_operations = if req.include_authorized_operations {
        -1
    } else {
        i32::MIN
    };

    if !is_coordinator_node(sdm).await {
        let groups = req
            .groups
            .iter()
            .map(|id| {
                DescribedGroup::default()
                    .with_group_id(id.clone())
                    .with_error_code(ResponseError::NotCoordinator.code())
                    .with_group_state(StrBytes::from_static_str("Dead"))
                    .with_authorized_operations(authorized_operations)
            })
            .collect();
        return Some(KafkaPacket::DescribeGroupsResponse(
            DescribeGroupsResponse::default().with_groups(groups),
        ));
    }

    let groups = req
        .groups
        .iter()
        .map(|id| match coordinator.describe_group(id.as_str()) {
            Some(info) => {
                let members = info
                    .members
                    .into_iter()
                    .map(|m| {
                        DescribedGroupMember::default()
                            .with_member_id(StrBytes::from(m.member_id))
                            .with_group_instance_id(m.group_instance_id.map(StrBytes::from))
                            .with_client_id(StrBytes::from(m.client_id))
                            .with_client_host(StrBytes::from_static_str(""))
                            .with_member_metadata(m.member_metadata)
                            .with_member_assignment(m.member_assignment)
                    })
                    .collect();
                DescribedGroup::default()
                    .with_group_id(id.clone())
                    .with_error_code(0)
                    .with_group_state(StrBytes::from(info.state))
                    .with_protocol_type(StrBytes::from(info.protocol_type))
                    .with_protocol_data(StrBytes::from(info.protocol_data))
                    .with_members(members)
                    .with_authorized_operations(authorized_operations)
            }
            None => DescribedGroup::default()
                .with_group_id(id.clone())
                .with_error_code(0)
                .with_group_state(StrBytes::from_static_str("Dead"))
                .with_authorized_operations(authorized_operations),
        })
        .collect();

    Some(KafkaPacket::DescribeGroupsResponse(
        DescribeGroupsResponse::default().with_groups(groups),
    ))
}

pub fn process_list_groups(
    coordinator: &GroupCoordinator,
    req: &ListGroupsRequest,
) -> Option<KafkaPacket> {
    let states_filter: Vec<String> = req.states_filter.iter().map(|s| s.to_lowercase()).collect();

    let groups = coordinator
        .list_groups()
        .into_iter()
        .filter(|g| states_filter.is_empty() || states_filter.contains(&g.state.to_lowercase()))
        .map(|g| {
            ListedGroup::default()
                .with_group_id(GroupId(StrBytes::from(g.group_id)))
                .with_protocol_type(StrBytes::from(g.protocol_type))
                .with_group_state(StrBytes::from(g.state))
                .with_group_type(StrBytes::from(g.group_type))
        })
        .collect();

    Some(KafkaPacket::ListGroupsResponse(
        ListGroupsResponse::default()
            .with_error_code(0)
            .with_groups(groups),
    ))
}

pub async fn process_delete_groups(
    coordinator: &GroupCoordinator,
    sdm: &Arc<StorageDriverManager>,
    req: &DeleteGroupsRequest,
) -> Option<KafkaPacket> {
    if !is_coordinator_node(sdm).await {
        let results = req
            .groups_names
            .iter()
            .map(|id| {
                DeletableGroupResult::default()
                    .with_group_id(id.clone())
                    .with_error_code(ResponseError::NotCoordinator.code())
            })
            .collect();
        return Some(KafkaPacket::DeleteGroupsResponse(
            DeleteGroupsResponse::default().with_results(results),
        ));
    }

    let mut results = Vec::with_capacity(req.groups_names.len());
    for id in &req.groups_names {
        let group_id = id.to_string();
        let mut code = coordinator.delete_group(&group_id);

        if code == 0 || code == ResponseError::GroupIdNotFound.code() {
            match delete_persisted_group_offsets(sdm, &group_id).await {
                Ok(had_offsets) => {
                    if code != 0 && had_offsets {
                        code = 0;
                    }
                }
                Err(e) => {
                    warn!(
                        "Kafka DeleteGroups failed to delete offsets for {}: {}",
                        group_id, e
                    );
                    code = ResponseError::UnknownServerError.code();
                }
            }
        }

        results.push(
            DeletableGroupResult::default()
                .with_group_id(id.clone())
                .with_error_code(code),
        );
    }

    Some(KafkaPacket::DeleteGroupsResponse(
        DeleteGroupsResponse::default().with_results(results),
    ))
}

fn join_group_error(code: i16) -> KafkaPacket {
    KafkaPacket::JoinGroupResponse(
        JoinGroupResponse::default()
            .with_error_code(code)
            .with_generation_id(-1),
    )
}

async fn delete_persisted_group_offsets(
    sdm: &Arc<StorageDriverManager>,
    group_id: &str,
) -> Result<bool, String> {
    let offsets = sdm
        .get_offset_by_group(get_tenant(), group_id)
        .await
        .map_err(|e| e.to_string())?;
    if offsets.is_empty() {
        return Ok(false);
    }
    let shard_names: Vec<String> = offsets.into_iter().map(|o| o.shard_name).collect();
    sdm.delete_group_offset(get_tenant(), group_id, &shard_names)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}
