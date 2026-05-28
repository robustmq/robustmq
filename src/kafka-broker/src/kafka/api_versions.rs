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
    api_versions_request::ApiVersionsRequest, api_versions_response::ApiVersion,
    create_topics_request::CreateTopicsRequest, delete_topics_request::DeleteTopicsRequest,
    describe_groups_request::DescribeGroupsRequest, fetch_request::FetchRequest,
    find_coordinator_request::FindCoordinatorRequest, heartbeat_request::HeartbeatRequest,
    join_group_request::JoinGroupRequest, leave_group_request::LeaveGroupRequest,
    list_groups_request::ListGroupsRequest, list_offsets_request::ListOffsetsRequest,
    metadata_request::MetadataRequest, offset_commit_request::OffsetCommitRequest,
    offset_fetch_request::OffsetFetchRequest, produce_request::ProduceRequest,
    sasl_authenticate_request::SaslAuthenticateRequest,
    sasl_handshake_request::SaslHandshakeRequest, sync_group_request::SyncGroupRequest, ApiKey,
    ApiVersionsResponse,
};
use kafka_protocol::protocol::Message;
use protocol::kafka::packet::KafkaPacket;

pub fn process_api_versions() -> Option<KafkaPacket> {
    let api_keys = vec![
        ApiVersion::default()
            .with_api_key(ApiKey::Produce as i16)
            .with_min_version(ProduceRequest::VERSIONS.min)
            .with_max_version(ProduceRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::Fetch as i16)
            .with_min_version(FetchRequest::VERSIONS.min)
            .with_max_version(FetchRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::ListOffsets as i16)
            .with_min_version(ListOffsetsRequest::VERSIONS.min)
            .with_max_version(ListOffsetsRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::Metadata as i16)
            .with_min_version(MetadataRequest::VERSIONS.min)
            .with_max_version(MetadataRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::OffsetCommit as i16)
            .with_min_version(OffsetCommitRequest::VERSIONS.min)
            .with_max_version(OffsetCommitRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::OffsetFetch as i16)
            .with_min_version(OffsetFetchRequest::VERSIONS.min)
            .with_max_version(OffsetFetchRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::FindCoordinator as i16)
            .with_min_version(FindCoordinatorRequest::VERSIONS.min)
            .with_max_version(FindCoordinatorRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::JoinGroup as i16)
            .with_min_version(JoinGroupRequest::VERSIONS.min)
            .with_max_version(JoinGroupRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::Heartbeat as i16)
            .with_min_version(HeartbeatRequest::VERSIONS.min)
            .with_max_version(HeartbeatRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::LeaveGroup as i16)
            .with_min_version(LeaveGroupRequest::VERSIONS.min)
            .with_max_version(LeaveGroupRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::SyncGroup as i16)
            .with_min_version(SyncGroupRequest::VERSIONS.min)
            .with_max_version(SyncGroupRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::DescribeGroups as i16)
            .with_min_version(DescribeGroupsRequest::VERSIONS.min)
            .with_max_version(DescribeGroupsRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::ListGroups as i16)
            .with_min_version(ListGroupsRequest::VERSIONS.min)
            .with_max_version(ListGroupsRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::SaslHandshake as i16)
            .with_min_version(SaslHandshakeRequest::VERSIONS.min)
            .with_max_version(SaslHandshakeRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::ApiVersions as i16)
            .with_min_version(ApiVersionsRequest::VERSIONS.min)
            .with_max_version(ApiVersionsRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::CreateTopics as i16)
            .with_min_version(CreateTopicsRequest::VERSIONS.min)
            .with_max_version(CreateTopicsRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::DeleteTopics as i16)
            .with_min_version(DeleteTopicsRequest::VERSIONS.min)
            .with_max_version(DeleteTopicsRequest::VERSIONS.max),
        ApiVersion::default()
            .with_api_key(ApiKey::SaslAuthenticate as i16)
            .with_min_version(SaslAuthenticateRequest::VERSIONS.min)
            .with_max_version(SaslAuthenticateRequest::VERSIONS.max),
    ];

    let resp = ApiVersionsResponse::default()
        .with_error_code(0)
        .with_api_keys(api_keys);

    Some(KafkaPacket::ApiVersionResponse(resp))
}
