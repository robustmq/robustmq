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

use crate::core::assignor::TopicMeta;
use crate::core::consumer_group_meta::topic_uuid;
use crate::core::consumer_heartbeat::ConsumerHeartbeatParams;
use crate::core::coordinator::GroupCoordinator;
use crate::core::coordinator_locator::is_coordinator_node;
use crate::handler::tenant::get_tenant;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::consumer_group_describe_response::{
    Assignment as DescribedAssignment, DescribedGroup, Member,
    TopicPartitions as DescribedTopicPartitions,
};
use kafka_protocol::messages::consumer_group_heartbeat_response::{Assignment, TopicPartitions};
use kafka_protocol::messages::{
    ConsumerGroupDescribeRequest, ConsumerGroupDescribeResponse, ConsumerGroupHeartbeatRequest,
    ConsumerGroupHeartbeatResponse, TopicName,
};
use kafka_protocol::protocol::StrBytes;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;
use uuid::Uuid;

pub async fn process_consumer_group_heartbeat(
    coordinator: &GroupCoordinator,
    sdm: &Arc<StorageDriverManager>,
    client_id: String,
    req: &ConsumerGroupHeartbeatRequest,
) -> Option<KafkaPacket> {
    if !is_coordinator_node(sdm).await {
        return Some(heartbeat_error_response(
            ResponseError::NotCoordinator.code(),
            "this node is not the group coordinator",
        ));
    }
    if req.subscribed_topic_regex.is_some() {
        return Some(heartbeat_error_response(
            ResponseError::InvalidRequest.code(),
            "subscribed topic regex is not supported yet",
        ));
    }

    let params = ConsumerHeartbeatParams {
        group_id: req.group_id.to_string(),
        member_id: req.member_id.to_string(),
        member_epoch: req.member_epoch,
        instance_id: req.instance_id.as_ref().map(|s| s.to_string()),
        rack_id: req.rack_id.as_ref().map(|s| s.to_string()),
        client_id,
        rebalance_timeout_ms: req.rebalance_timeout_ms,
        subscribed_topics: req
            .subscribed_topic_names
            .as_ref()
            .map(|names| names.iter().map(|n| n.to_string()).collect()),
        server_assignor: req.server_assignor.as_ref().map(|s| s.to_string()),
        owned: req.topic_partitions.as_ref().map(|tps| {
            tps.iter()
                .map(|tp| (tp.topic_id, tp.partitions.clone()))
                .collect()
        }),
    };

    let broker_cache = sdm.broker_cache.clone();
    let tenant = get_tenant();
    let resolve_topic = move |name: &str| -> Option<TopicMeta> {
        broker_cache
            .get_topic_by_name(tenant, name)
            .map(|t| TopicMeta {
                topic_id: topic_uuid(tenant, name),
                partitions: t.partition,
            })
    };

    let result = coordinator.consumer_heartbeat(params, &resolve_topic);

    Some(KafkaPacket::ConsumerGroupHeartbeatResponse(
        ConsumerGroupHeartbeatResponse::default()
            .with_error_code(result.error_code)
            .with_error_message(result.error_message.map(StrBytes::from))
            .with_member_id(Some(StrBytes::from(result.member_id)))
            .with_member_epoch(result.member_epoch)
            .with_heartbeat_interval_ms(coordinator.consumer_heartbeat_interval_ms())
            .with_assignment(result.assignment.map(to_wire_assignment)),
    ))
}

pub async fn process_consumer_group_describe(
    coordinator: &GroupCoordinator,
    sdm: &Arc<StorageDriverManager>,
    req: &ConsumerGroupDescribeRequest,
) -> Option<KafkaPacket> {
    let authorized_operations = if req.include_authorized_operations {
        -1
    } else {
        i32::MIN
    };

    if !is_coordinator_node(sdm).await {
        let groups = req
            .group_ids
            .iter()
            .map(|id| {
                DescribedGroup::default()
                    .with_group_id(id.clone())
                    .with_error_code(ResponseError::NotCoordinator.code())
                    .with_group_state(StrBytes::from_static_str("Dead"))
                    .with_authorized_operations(authorized_operations)
            })
            .collect();
        return Some(KafkaPacket::ConsumerGroupDescribeResponse(
            ConsumerGroupDescribeResponse::default().with_groups(groups),
        ));
    }

    let groups = req
        .group_ids
        .iter()
        .map(
            |id| match coordinator.describe_consumer_group(id.as_str(), get_tenant()) {
                Some(info) => {
                    let members = info
                        .members
                        .iter()
                        .map(|m| {
                            Member::default()
                                .with_member_id(StrBytes::from(m.member_id.clone()))
                                .with_instance_id(m.instance_id.clone().map(StrBytes::from))
                                .with_rack_id(m.rack_id.clone().map(StrBytes::from))
                                .with_member_epoch(m.member_epoch)
                                .with_client_id(StrBytes::from(m.client_id.clone()))
                                .with_client_host(StrBytes::from_static_str(""))
                                .with_subscribed_topic_names(
                                    m.subscribed
                                        .iter()
                                        .map(|n| TopicName(StrBytes::from(n.clone())))
                                        .collect(),
                                )
                                .with_assignment(to_described_assignment(
                                    &m.assignment,
                                    &info.topic_names,
                                ))
                                .with_target_assignment(to_described_assignment(
                                    &m.target_assignment,
                                    &info.topic_names,
                                ))
                        })
                        .collect();
                    DescribedGroup::default()
                        .with_group_id(id.clone())
                        .with_error_code(0)
                        .with_group_state(StrBytes::from(info.state))
                        .with_group_epoch(info.group_epoch)
                        .with_assignment_epoch(info.assignment_epoch)
                        .with_assignor_name(StrBytes::from(info.assignor))
                        .with_members(members)
                        .with_authorized_operations(authorized_operations)
                }
                None => DescribedGroup::default()
                    .with_group_id(id.clone())
                    .with_error_code(ResponseError::GroupIdNotFound.code())
                    .with_group_state(StrBytes::from_static_str("Dead"))
                    .with_authorized_operations(authorized_operations),
            },
        )
        .collect();

    Some(KafkaPacket::ConsumerGroupDescribeResponse(
        ConsumerGroupDescribeResponse::default().with_groups(groups),
    ))
}

fn heartbeat_error_response(code: i16, message: &str) -> KafkaPacket {
    KafkaPacket::ConsumerGroupHeartbeatResponse(
        ConsumerGroupHeartbeatResponse::default()
            .with_error_code(code)
            .with_error_message(Some(StrBytes::from(message.to_string()))),
    )
}

fn to_wire_assignment(assignment: HashMap<Uuid, Vec<i32>>) -> Assignment {
    let mut topic_partitions: Vec<TopicPartitions> = assignment
        .into_iter()
        .map(|(topic_id, partitions)| {
            TopicPartitions::default()
                .with_topic_id(topic_id)
                .with_partitions(partitions)
        })
        .collect();
    topic_partitions.sort_by_key(|t| t.topic_id);
    Assignment::default().with_topic_partitions(topic_partitions)
}

fn to_described_assignment(
    assignment: &HashMap<Uuid, Vec<i32>>,
    topic_names: &HashMap<Uuid, String>,
) -> DescribedAssignment {
    let mut topic_partitions: Vec<DescribedTopicPartitions> = assignment
        .iter()
        .map(|(topic_id, partitions)| {
            DescribedTopicPartitions::default()
                .with_topic_id(*topic_id)
                .with_topic_name(TopicName(StrBytes::from(
                    topic_names.get(topic_id).cloned().unwrap_or_default(),
                )))
                .with_partitions(partitions.clone())
        })
        .collect();
    topic_partitions.sort_by_key(|t| t.topic_id);
    DescribedAssignment::default().with_topic_partitions(topic_partitions)
}
