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

use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::alter_partition_reassignments_response::AlterPartitionReassignmentsResponse;
use kafka_protocol::messages::alter_replica_log_dirs_response::{
    AlterReplicaLogDirPartitionResult, AlterReplicaLogDirTopicResult, AlterReplicaLogDirsResponse,
};
use kafka_protocol::messages::describe_log_dirs_response::DescribeLogDirsResponse;
use kafka_protocol::messages::elect_leaders_response::ElectLeadersResponse;
use kafka_protocol::messages::list_partition_reassignments_response::ListPartitionReassignmentsResponse;
use kafka_protocol::messages::update_features_response::UpdatableFeatureResult;
use kafka_protocol::messages::{
    AlterPartitionReassignmentsRequest, AlterReplicaLogDirsRequest, DescribeLogDirsRequest,
    DescribeProducersRequest, ElectLeadersRequest, ListPartitionReassignmentsRequest,
    UpdateFeaturesRequest, UpdateFeaturesResponse,
};
use kafka_protocol::protocol::StrBytes;
use protocol::kafka::packet::KafkaPacket;
use tracing::warn;

const NOT_SUPPORTED_MESSAGE: &str =
    "This operation is not supported by RobustMQ's storage architecture";

// RobustMQ segments are append-only and self-contained: a segment is
// assigned one data directory at creation and never moves. When a directory
// fills up, the *next* segment simply lands on a different directory picked
// automatically — there is no "existing segment outgrew its disk, move it"
// scenario for AlterReplicaLogDirs to address, so per-replica manual disk
// placement has no equivalent concept here.
pub fn process_alter_replica_log_dirs(req: &AlterReplicaLogDirsRequest) -> Option<KafkaPacket> {
    warn!("Kafka AlterReplicaLogDirs is not supported: RobustMQ assigns segment storage directories automatically");

    let mut partitions_by_topic: HashMap<String, Vec<i32>> = HashMap::new();
    for dir in &req.dirs {
        for topic in &dir.topics {
            partitions_by_topic
                .entry(topic.name.to_string())
                .or_default()
                .extend(topic.partitions.iter().copied());
        }
    }

    let results = partitions_by_topic
        .into_iter()
        .map(|(name, partitions)| {
            let partitions = partitions
                .into_iter()
                .map(|partition_index| {
                    AlterReplicaLogDirPartitionResult::default()
                        .with_partition_index(partition_index)
                        .with_error_code(ResponseError::LogDirNotFound.code())
                })
                .collect();
            AlterReplicaLogDirTopicResult::default()
                .with_topic_name(kafka_protocol::messages::TopicName(StrBytes::from(name)))
                .with_partitions(partitions)
        })
        .collect();

    Some(KafkaPacket::AlterReplicaLogDirsResponse(
        AlterReplicaLogDirsResponse::default().with_results(results),
    ))
}

// Companion query to AlterReplicaLogDirs — same reasoning: segment-to-directory
// placement is fully automatic and not something RobustMQ exposes per-replica
// disk usage for.
pub fn process_describe_log_dirs(_req: &DescribeLogDirsRequest) -> Option<KafkaPacket> {
    warn!("Kafka DescribeLogDirs is not supported: RobustMQ does not expose per-disk segment placement");
    Some(KafkaPacket::DescribeLogDirsResponse(
        DescribeLogDirsResponse::default()
            .with_error_code(ResponseError::UnsupportedVersion.code()),
    ))
}

// Leader placement is fully automatic (ISR-based, with a background
// preferred-replica rebalancer — see storage-engine's leader-rebalance
// machinery), so there is no manual/operator-triggered election lever to
// wire up here.
pub fn process_elect_leaders(_req: &ElectLeadersRequest) -> Option<KafkaPacket> {
    warn!("Kafka ElectLeaders is not supported: leader placement is fully automatic in RobustMQ");
    Some(KafkaPacket::ElectLeadersResponse(
        ElectLeadersResponse::default().with_error_code(ResponseError::UnsupportedVersion.code()),
    ))
}

// General-purpose partition-to-node reassignment (moving a partition's
// replicas to a different set of nodes on demand) is an operational concern
// that, if implemented, belongs in the storage layer's own replica-placement
// machinery — not in the Kafka protocol surface. Kafka clients/tools should
// not expect this to ever be wired up here.
pub fn process_alter_partition_reassignments(
    _req: &AlterPartitionReassignmentsRequest,
) -> Option<KafkaPacket> {
    warn!("Kafka AlterPartitionReassignments is not supported: replica placement is managed internally, not via the Kafka protocol");
    Some(KafkaPacket::AlterPartitionReassignmentsResponse(
        AlterPartitionReassignmentsResponse::default()
            .with_error_code(ResponseError::UnsupportedVersion.code())
            .with_error_message(Some(StrBytes::from_static_str(NOT_SUPPORTED_MESSAGE))),
    ))
}

// Companion query to AlterPartitionReassignments — same reasoning.
pub fn process_list_partition_reassignments(
    _req: &ListPartitionReassignmentsRequest,
) -> Option<KafkaPacket> {
    warn!("Kafka ListPartitionReassignments is not supported: replica placement is managed internally, not via the Kafka protocol");
    Some(KafkaPacket::ListPartitionReassignmentsResponse(
        ListPartitionReassignmentsResponse::default()
            .with_error_code(ResponseError::UnsupportedVersion.code())
            .with_error_message(Some(StrBytes::from_static_str(NOT_SUPPORTED_MESSAGE))),
    ))
}

// RobustMQ has no KRaft-style feature-versioning metadata to update against,
// so this is a deliberate not-yet-supported response rather than a stub —
// every requested feature is reported as failed instead of silently dropping
// the request.
pub fn process_update_features(req: &UpdateFeaturesRequest) -> Option<KafkaPacket> {
    let feature_names: Vec<String> = req
        .feature_updates
        .iter()
        .map(|f| f.feature.to_string())
        .collect();
    warn!(
        "Kafka UpdateFeatures is not supported yet; rejecting request for features: {:?}",
        feature_names
    );

    let error_message = Some(kafka_protocol::protocol::StrBytes::from_static_str(
        "UpdateFeatures is not supported by this broker",
    ));
    let results = req
        .feature_updates
        .iter()
        .map(|f| {
            UpdatableFeatureResult::default()
                .with_feature(f.feature.clone())
                .with_error_code(ResponseError::FeatureUpdateFailed.code())
                .with_error_message(error_message.clone())
        })
        .collect();

    Some(KafkaPacket::UpdateFeaturesResponse(
        UpdateFeaturesResponse::default()
            .with_error_code(ResponseError::FeatureUpdateFailed.code())
            .with_error_message(error_message)
            .with_results(results),
    ))
}

pub fn process_describe_producers(_req: &DescribeProducersRequest) -> Option<KafkaPacket> {
    None
}
