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

use std::collections::HashMap;
use std::sync::Arc;

use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::join_group_response::JoinGroupResponseMember;
use kafka_protocol::messages::offset_commit_response::{
    OffsetCommitResponsePartition, OffsetCommitResponseTopic,
};
use kafka_protocol::messages::offset_fetch_response::{
    OffsetFetchResponsePartition, OffsetFetchResponseTopic,
};
use kafka_protocol::messages::{
    DeleteGroupsRequest, DescribeGroupsRequest, HeartbeatRequest, HeartbeatResponse,
    JoinGroupRequest, JoinGroupResponse, LeaveGroupRequest, LeaveGroupResponse, ListGroupsRequest,
    ListGroupsResponse, OffsetCommitRequest, OffsetCommitResponse, OffsetDeleteRequest,
    OffsetFetchRequest, OffsetFetchResponse, SyncGroupRequest, SyncGroupResponse,
};
use metadata_struct::tenant::DEFAULT_TENANT;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

// A committed_offset of -1 means "don't commit this partition" (client opted out).
const NO_COMMIT_OFFSET: i64 = -1;

pub async fn process_offset_commit(
    sdm: &Arc<StorageDriverManager>,
    req: &OffsetCommitRequest,
) -> Option<KafkaPacket> {
    let group_id = req.group_id.to_string();

    // Resolve each requested partition to its shard_name up front, so we can
    // commit every valid shard in a single call and still report a per-partition
    // error code (UnknownTopicOrPartition) for ones that don't resolve.
    let mut shard_offsets = HashMap::new();
    let mut partition_shards: Vec<Vec<Option<String>>> = Vec::with_capacity(req.topics.len());

    for t in &req.topics {
        let topic_name = t.name.to_string();
        let topic = sdm
            .broker_cache
            .get_topic_by_name(DEFAULT_TENANT, &topic_name);

        let shards = t
            .partitions
            .iter()
            .map(|p| {
                let shard_name = topic
                    .as_ref()?
                    .storage_name_list
                    .get(&(p.partition_index as u32))?
                    .clone();
                if p.committed_offset != NO_COMMIT_OFFSET {
                    shard_offsets.insert(shard_name.clone(), p.committed_offset as u64);
                }
                Some(shard_name)
            })
            .collect();
        partition_shards.push(shards);
    }

    let commit_error_code = if shard_offsets.is_empty() {
        0
    } else if let Err(e) = sdm
        .commit_offset(DEFAULT_TENANT, &group_id, &shard_offsets)
        .await
    {
        warn!(
            "Kafka OffsetCommit storage error for group {}: {}",
            group_id, e
        );
        ResponseError::UnknownServerError.code()
    } else {
        0
    };

    let topics = req
        .topics
        .iter()
        .zip(partition_shards)
        .map(|(t, shards)| {
            let partitions = t
                .partitions
                .iter()
                .zip(shards)
                .map(|(p, shard_name)| {
                    let error_code = match shard_name {
                        None => ResponseError::UnknownTopicOrPartition.code(),
                        Some(_) if p.committed_offset == NO_COMMIT_OFFSET => 0,
                        Some(_) => commit_error_code,
                    };
                    OffsetCommitResponsePartition::default()
                        .with_partition_index(p.partition_index)
                        .with_error_code(error_code)
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
