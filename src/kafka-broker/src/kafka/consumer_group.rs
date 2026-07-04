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

use crate::kafka::metadata::split_host_port;
use common_config::broker::broker_config;
use grpc_clients::meta::kafka::call::get_coordinator_leader;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::find_coordinator_response::Coordinator;
use kafka_protocol::messages::join_group_response::JoinGroupResponseMember;
use kafka_protocol::messages::{
    DeleteGroupsRequest, DescribeGroupsRequest, FindCoordinatorRequest, FindCoordinatorResponse,
    HeartbeatRequest, HeartbeatResponse, JoinGroupRequest, JoinGroupResponse, LeaveGroupRequest,
    LeaveGroupResponse, ListGroupsRequest, ListGroupsResponse, SyncGroupRequest, SyncGroupResponse,
};
use kafka_protocol::protocol::StrBytes;
use protocol::kafka::packet::KafkaPacket;
use protocol::meta::meta_service_kafka::GetCoordinatorLeaderRequest;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

// Kafka FindCoordinator key_type: 0=group, 1=transaction, 2=share.
const KEY_TYPE_GROUP: i8 = 0;

// Every consumer group is coordinated by the node hosting the metadata-raft leader:
// group state lives in meta/raft, so co-locating the coordinator there keeps state
// access local and lets coordinator failover ride raft leader election for free.
// Returns (node_id, host, port) or a Kafka error code.
async fn resolve_group_coordinator(
    sdm: &Arc<StorageDriverManager>,
) -> Result<(i32, String, i32), i16> {
    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();

    let reply = get_coordinator_leader(client_pool, &addrs, GetCoordinatorLeaderRequest {})
        .await
        .map_err(|e| {
            warn!(
                "Kafka FindCoordinator: failed to get coordinator leader: {}",
                e
            );
            ResponseError::CoordinatorNotAvailable.code()
        })?;
    if !reply.has_leader {
        return Err(ResponseError::CoordinatorNotAvailable.code());
    }

    let node = sdm
        .broker_cache
        .node_lists
        .get(&reply.leader_node_id)
        .map(|n| n.clone())
        .ok_or_else(|| ResponseError::CoordinatorNotAvailable.code())?;
    let (host, port) = split_host_port(&node.extend.kafka.tcp_addr)
        .ok_or_else(|| ResponseError::CoordinatorNotAvailable.code())?;

    Ok((node.node_id as i32, host, port))
}

pub async fn process_find_coordinator(
    sdm: &Arc<StorageDriverManager>,
    req: &FindCoordinatorRequest,
) -> Option<KafkaPacket> {
    // Only consumer-group coordinators are supported; transaction/share are not.
    let resolved = if req.key_type == KEY_TYPE_GROUP {
        resolve_group_coordinator(sdm).await
    } else {
        Err(ResponseError::CoordinatorNotAvailable.code())
    };

    let (error_code, node_id, host, port) = match resolved {
        Ok((node_id, host, port)) => (0, node_id, host, port),
        Err(code) => (code, -1, String::new(), -1),
    };

    // v4+ carries a per-key `coordinators` list; v0-3 carries the single top-level
    // node/host/port. Fill both; the codec emits only the negotiated version's fields.
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

pub fn process_join_group(req: &JoinGroupRequest) -> Option<KafkaPacket> {
    // Make this consumer both the leader and sole member.
    let member_id = if req.member_id.is_empty() {
        "robustmq-member-1".into()
    } else {
        req.member_id.clone()
    };

    // Pick the first proposed protocol.
    let protocol_name = req
        .protocols
        .first()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "range".into());

    // Echo back metadata from the first protocol as the member's metadata.
    let metadata = req
        .protocols
        .first()
        .map(|p| p.metadata.clone())
        .unwrap_or_default();

    let members = vec![JoinGroupResponseMember::default()
        .with_member_id(member_id.clone())
        .with_metadata(metadata)];

    Some(KafkaPacket::JoinGroupResponse(
        JoinGroupResponse::default()
            .with_error_code(0)
            .with_generation_id(1)
            .with_protocol_type(Some("consumer".into()))
            .with_protocol_name(Some(protocol_name))
            .with_leader(member_id.clone())
            .with_member_id(member_id)
            .with_members(members),
    ))
}

pub fn process_heartbeat(_req: &HeartbeatRequest) -> Option<KafkaPacket> {
    Some(KafkaPacket::HeartbeatResponse(
        HeartbeatResponse::default().with_error_code(0),
    ))
}

pub fn process_leave_group(_req: &LeaveGroupRequest) -> Option<KafkaPacket> {
    Some(KafkaPacket::LeaveGroupResponse(
        LeaveGroupResponse::default().with_error_code(0),
    ))
}

pub fn process_sync_group(req: &SyncGroupRequest) -> Option<KafkaPacket> {
    // Echo back the assignment sent by the leader (the only member).
    let assignment = req
        .assignments
        .first()
        .map(|a| a.assignment.clone())
        .unwrap_or_default();

    Some(KafkaPacket::SyncGroupResponse(
        SyncGroupResponse::default()
            .with_error_code(0)
            .with_protocol_type(Some("consumer".into()))
            .with_protocol_name(Some(
                req.protocol_name.clone().unwrap_or_else(|| "range".into()),
            ))
            .with_assignment(assignment),
    ))
}

pub fn process_describe_groups(_req: &DescribeGroupsRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_list_groups(_req: &ListGroupsRequest) -> Option<KafkaPacket> {
    Some(KafkaPacket::ListGroupsResponse(
        ListGroupsResponse::default()
            .with_error_code(0)
            .with_groups(vec![]),
    ))
}

pub fn process_delete_groups(_req: &DeleteGroupsRequest) -> Option<KafkaPacket> {
    None
}
