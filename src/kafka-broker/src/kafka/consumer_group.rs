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

use kafka_protocol::messages::join_group_response::JoinGroupResponseMember;
use kafka_protocol::messages::offset_fetch_response::{
    OffsetFetchResponsePartition, OffsetFetchResponseTopic,
};
use kafka_protocol::messages::{
    DeleteGroupsRequest, DescribeGroupsRequest, HeartbeatRequest, HeartbeatResponse,
    JoinGroupRequest, JoinGroupResponse, LeaveGroupRequest, LeaveGroupResponse, ListGroupsRequest,
    ListGroupsResponse, OffsetCommitRequest, OffsetDeleteRequest, OffsetFetchRequest,
    OffsetFetchResponse, SyncGroupRequest, SyncGroupResponse,
};
use protocol::kafka::packet::KafkaPacket;

pub fn process_offset_commit(req: &OffsetCommitRequest) -> Option<KafkaPacket> {
    use kafka_protocol::messages::offset_commit_response::{
        OffsetCommitResponsePartition, OffsetCommitResponseTopic,
    };
    use kafka_protocol::messages::OffsetCommitResponse;

    let topics = req
        .topics
        .iter()
        .map(|t| {
            let partitions = t
                .partitions
                .iter()
                .map(|p| {
                    OffsetCommitResponsePartition::default()
                        .with_partition_index(p.partition_index)
                        .with_error_code(0)
                })
                .collect();
            OffsetCommitResponseTopic::default()
                .with_name(t.name.clone())
                .with_partitions(partitions)
        })
        .collect();

    Some(KafkaPacket::OffsetCommitResponse(
        OffsetCommitResponse::default().with_topics(topics),
    ))
}

pub fn process_offset_fetch(req: &OffsetFetchRequest) -> Option<KafkaPacket> {
    use kafka_protocol::messages::offset_fetch_response::{
        OffsetFetchResponseGroup, OffsetFetchResponsePartitions, OffsetFetchResponseTopics,
    };

    // v8+ uses `groups` field; older versions use `topics` field directly.
    if !req.groups.is_empty() {
        // New format (v8+): respond per-group
        let groups = req
            .groups
            .iter()
            .map(|g| {
                let topics = g
                    .topics
                    .iter()
                    .flatten()
                    .map(|t| {
                        let partitions = t
                            .partition_indexes
                            .iter()
                            .map(|&p| {
                                OffsetFetchResponsePartitions::default()
                                    .with_partition_index(p)
                                    .with_committed_offset(-1)
                                    .with_error_code(0)
                            })
                            .collect();
                        OffsetFetchResponseTopics::default()
                            .with_name(t.name.clone())
                            .with_partitions(partitions)
                    })
                    .collect();
                OffsetFetchResponseGroup::default()
                    .with_group_id(g.group_id.clone())
                    .with_topics(topics)
                    .with_error_code(0)
            })
            .collect();

        return Some(KafkaPacket::OffsetFetchResponse(
            OffsetFetchResponse::default().with_groups(groups),
        ));
    }

    // Old format: topics directly on request
    let topics = req
        .topics
        .iter()
        .flatten()
        .map(|t| {
            let partitions = t
                .partition_indexes
                .iter()
                .map(|&p| {
                    OffsetFetchResponsePartition::default()
                        .with_partition_index(p)
                        .with_committed_offset(-1)
                        .with_error_code(0)
                })
                .collect();
            OffsetFetchResponseTopic::default()
                .with_name(t.name.clone())
                .with_partitions(partitions)
        })
        .collect();

    Some(KafkaPacket::OffsetFetchResponse(
        OffsetFetchResponse::default()
            .with_topics(topics)
            .with_error_code(0),
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

pub fn process_offset_delete(_req: &OffsetDeleteRequest) -> Option<KafkaPacket> {
    None
}
