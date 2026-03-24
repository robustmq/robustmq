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

use kafka_protocol::messages::{
    AlterPartitionReassignmentsRequest, AlterReplicaLogDirsRequest, DescribeClusterRequest,
    DescribeLogDirsRequest, DescribeProducersRequest, DescribeTopicPartitionsRequest,
    ElectLeadersRequest, ListPartitionReassignmentsRequest, OffsetForLeaderEpochRequest,
    UpdateFeaturesRequest,
};
use protocol::kafka::packet::KafkaPacket;

pub fn process_offset_for_leader_epoch(_req: &OffsetForLeaderEpochRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_alter_replica_log_dirs(_req: &AlterReplicaLogDirsRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_describe_log_dirs(_req: &DescribeLogDirsRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_elect_leaders(_req: &ElectLeadersRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_alter_partition_reassignments(
    _req: &AlterPartitionReassignmentsRequest,
) -> Option<KafkaPacket> {
    None
}

pub fn process_list_partition_reassignments(
    _req: &ListPartitionReassignmentsRequest,
) -> Option<KafkaPacket> {
    None
}

pub fn process_update_features(_req: &UpdateFeaturesRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_describe_cluster(_req: &DescribeClusterRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_describe_producers(_req: &DescribeProducersRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_describe_topic_partitions(
    _req: &DescribeTopicPartitionsRequest,
) -> Option<KafkaPacket> {
    None
}
