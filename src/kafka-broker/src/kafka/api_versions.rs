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

use kafka_protocol::messages::{api_versions_response::ApiVersion, ApiKey, ApiVersionsResponse};
use protocol::kafka::packet::KafkaPacket;

// Version ranges are aligned with Redpanda's declared min_supported/max_supported
// values (src/v/kafka/server/handlers/api_versions.cc, dev branch).
// Only APIs Redpanda exposes to clients are listed here; internal KRaft/broker
// APIs (e.g. Vote, BrokerRegistration) are intentionally excluded.
fn v(key: ApiKey, min: i16, max: i16) -> ApiVersion {
    ApiVersion::default()
        .with_api_key(key as i16)
        .with_min_version(min)
        .with_max_version(max)
}

pub fn process_api_versions() -> Option<KafkaPacket> {
    let api_keys = vec![
        // ── Core produce / consume ────────────────────────────────────────
        v(ApiKey::Produce, 0, 7),
        v(ApiKey::Fetch, 4, 13),
        v(ApiKey::ListOffsets, 0, 6),
        v(ApiKey::Metadata, 0, 12),
        // ── Consumer group ───────────────────────────────────────────────
        v(ApiKey::OffsetCommit, 1, 8),
        v(ApiKey::OffsetFetch, 1, 8),
        v(ApiKey::FindCoordinator, 0, 4),
        v(ApiKey::JoinGroup, 0, 6),
        v(ApiKey::Heartbeat, 0, 4),
        v(ApiKey::LeaveGroup, 0, 4),
        v(ApiKey::SyncGroup, 0, 4),
        v(ApiKey::DescribeGroups, 0, 5),
        v(ApiKey::ListGroups, 0, 4),
        v(ApiKey::DeleteGroups, 0, 2),
        v(ApiKey::OffsetDelete, 0, 0),
        // ── Consumer group (KIP-848) ─────────────────────────────────────
        v(ApiKey::ConsumerGroupHeartbeat, 0, 1),
        v(ApiKey::ConsumerGroupDescribe, 0, 1),
        // ── Auth ─────────────────────────────────────────────────────────
        // Only v1 handshake is offered: v0 carried SASL tokens as raw bytes
        // outside the Kafka framing, which our network layer cannot parse.
        v(ApiKey::SaslHandshake, 1, 1),
        v(ApiKey::SaslAuthenticate, 0, 2),
        // ── API negotiation ───────────────────────────────────────────────
        v(ApiKey::ApiVersions, 0, 4),
        // ── Topic admin ───────────────────────────────────────────────────
        v(ApiKey::CreateTopics, 0, 7),
        v(ApiKey::DeleteTopics, 0, 6),
        v(ApiKey::DeleteRecords, 0, 2),
        v(ApiKey::CreatePartitions, 0, 3),
        // ── Config admin ─────────────────────────────────────────────────
        v(ApiKey::DescribeConfigs, 0, 4),
        v(ApiKey::AlterConfigs, 0, 2),
        v(ApiKey::IncrementalAlterConfigs, 0, 1),
        // ── Replica / log admin ───────────────────────────────────────────
        v(ApiKey::DescribeLogDirs, 0, 2),
        v(ApiKey::AlterPartitionReassignments, 0, 0),
        v(ApiKey::ListPartitionReassignments, 0, 0),
        // ── Idempotent producer ───────────────────────────────────────────
        v(ApiKey::InitProducerId, 0, 3),
        // ── Transactions ─────────────────────────────────────────────────
        v(ApiKey::AddPartitionsToTxn, 0, 3),
        v(ApiKey::AddOffsetsToTxn, 0, 1),
        v(ApiKey::EndTxn, 0, 3),
        v(ApiKey::TxnOffsetCommit, 0, 3),
        v(ApiKey::DescribeTransactions, 0, 0),
        v(ApiKey::ListTransactions, 0, 0),
        v(ApiKey::DescribeProducers, 0, 0),
        // ── ACL ──────────────────────────────────────────────────────────
        v(ApiKey::DescribeAcls, 0, 2),
        v(ApiKey::CreateAcls, 0, 2),
        v(ApiKey::DeleteAcls, 0, 2),
        // ── Quotas ───────────────────────────────────────────────────────
        v(ApiKey::DescribeClientQuotas, 0, 1),
        v(ApiKey::AlterClientQuotas, 0, 1),
        // ── SCRAM ────────────────────────────────────────────────────────
        v(ApiKey::DescribeUserScramCredentials, 0, 0),
        v(ApiKey::AlterUserScramCredentials, 0, 0),
        // ── Cluster ──────────────────────────────────────────────────────
        v(ApiKey::DescribeCluster, 0, 0),
    ];

    let resp = ApiVersionsResponse::default()
        .with_error_code(0)
        .with_api_keys(api_keys);

    Some(KafkaPacket::ApiVersionResponse(resp))
}
