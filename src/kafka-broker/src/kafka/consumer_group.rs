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
    DeleteGroupsRequest, DescribeGroupsRequest, FindCoordinatorRequest, HeartbeatRequest,
    JoinGroupRequest, LeaveGroupRequest, ListGroupsRequest, OffsetCommitRequest,
    OffsetDeleteRequest, OffsetFetchRequest, SyncGroupRequest,
};
use protocol::kafka::packet::KafkaPacket;

pub fn process_offset_commit(_req: &OffsetCommitRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_offset_fetch(_req: &OffsetFetchRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_find_coordinator(_req: &FindCoordinatorRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_join_group(_req: &JoinGroupRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_heartbeat(_req: &HeartbeatRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_leave_group(_req: &LeaveGroupRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_sync_group(_req: &SyncGroupRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_describe_groups(_req: &DescribeGroupsRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_list_groups(_req: &ListGroupsRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_delete_groups(_req: &DeleteGroupsRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_offset_delete(_req: &OffsetDeleteRequest) -> Option<KafkaPacket> {
    None
}
