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
    AlterShareGroupOffsetsRequest, DeleteShareGroupOffsetsRequest,
    DescribeShareGroupOffsetsRequest, ShareAcknowledgeRequest, ShareFetchRequest,
    ShareGroupDescribeRequest, ShareGroupHeartbeatRequest,
};
use protocol::kafka::packet::KafkaPacket;

pub fn process_share_group_heartbeat(_req: &ShareGroupHeartbeatRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_share_group_describe(_req: &ShareGroupDescribeRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_share_fetch(_req: &ShareFetchRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_share_acknowledge(_req: &ShareAcknowledgeRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_describe_share_group_offsets(
    _req: &DescribeShareGroupOffsetsRequest,
) -> Option<KafkaPacket> {
    None
}

pub fn process_alter_share_group_offsets(
    _req: &AlterShareGroupOffsetsRequest,
) -> Option<KafkaPacket> {
    None
}

pub fn process_delete_share_group_offsets(
    _req: &DeleteShareGroupOffsetsRequest,
) -> Option<KafkaPacket> {
    None
}
