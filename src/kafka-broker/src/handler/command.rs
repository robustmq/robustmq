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

use async_trait::async_trait;
use dashmap::DashMap;
use kafka_protocol::messages::fetch_response::{FetchableTopicResponse, PartitionData};
use kafka_protocol::messages::find_coordinator_response::Coordinator;
use kafka_protocol::messages::metadata_response::{
    MetadataResponseBroker, MetadataResponsePartition, MetadataResponseTopic,
};
use kafka_protocol::messages::{
    api_versions_request::ApiVersionsRequest, api_versions_response::ApiVersion,
    create_topics_request::CreateTopicsRequest, delete_topics_request::DeleteTopicsRequest,
    describe_groups_request::DescribeGroupsRequest, fetch_request::FetchRequest,
    find_coordinator_request::FindCoordinatorRequest,
    find_coordinator_response::FindCoordinatorResponse, heartbeat_request::HeartbeatRequest,
    join_group_request::JoinGroupRequest, leave_group_request::LeaveGroupRequest,
    list_groups_request::ListGroupsRequest, list_offsets_request::ListOffsetsRequest,
    metadata_request::MetadataRequest, metadata_response::MetadataResponse,
    offset_commit_request::OffsetCommitRequest, offset_fetch_request::OffsetFetchRequest,
    produce_request::ProduceRequest, sasl_authenticate_request::SaslAuthenticateRequest,
    sasl_handshake_request::SaslHandshakeRequest, sync_group_request::SyncGroupRequest, ApiKey,
    ApiVersionsResponse, FetchResponse, ResponseHeader,
};
use kafka_protocol::protocol::Message;
use kafka_protocol::records::{
    Compression, Record, RecordBatchEncoder, RecordEncodeOptions, TimestampType,
};
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::connection::NetworkConnection;
use metadata_struct::tenant::DEFAULT_TENANT;
use network_server::command::Command;
use network_server::common::packet::ResponsePackage;
use protocol::kafka::packet::{KafkaHeader, KafkaPacket, KafkaPacketWrapper};
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

use crate::kafka::{
    acl, admin, auth, config, consumer_group, consumer_group_next, core, delegation_token, quota,
    share_group, telemetry, topic, transaction,
};

#[derive(Clone)]
pub struct KafkaHandlerCommand {
    storage_driver_manager: Option<Arc<StorageDriverManager>>,
    // (connection_id, topic) -> per-shard offsets
    shard_offsets: Arc<DashMap<(u64, String), HashMap<String, u64>>>,
}

impl KafkaHandlerCommand {
    pub fn new() -> Self {
        KafkaHandlerCommand {
            storage_driver_manager: None,
            shard_offsets: Arc::new(DashMap::new()),
        }
    }

    pub fn new_with_storage(storage_driver_manager: Arc<StorageDriverManager>) -> Self {
        KafkaHandlerCommand {
            storage_driver_manager: Some(storage_driver_manager),
            shard_offsets: Arc::new(DashMap::new()),
        }
    }
}

impl Default for KafkaHandlerCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for KafkaHandlerCommand {
    async fn apply(
        &self,
        tcp_connection: &NetworkConnection,
        _addr: &SocketAddr,
        robust_packet: &RobustMQPacket,
    ) -> Option<ResponsePackage> {
        let wrapper = robust_packet.get_kafka_packet()?;
        let connection_id = tcp_connection.connection_id;

        let correlation_id = match &wrapper.header {
            KafkaHeader::Request(h) => h.correlation_id,
            KafkaHeader::Response(_) => return None,
        };

        let resp_packet = match &wrapper.packet {
            // Core Data Plane
            KafkaPacket::ProduceReq(req) => core::process_produce(req),
            KafkaPacket::FetchReq(req) => self.process_fetch(req, connection_id).await,
            KafkaPacket::ListOffsetsReq(req) => core::process_list_offsets(req),
            KafkaPacket::MetadataReq(req) => self.process_metadata(req),
            // Consumer Group Management
            KafkaPacket::OffsetCommitReq(req) => consumer_group::process_offset_commit(req),
            KafkaPacket::OffsetFetchReq(req) => consumer_group::process_offset_fetch(req),
            KafkaPacket::FindCoordinatorReq(req) => self.process_find_coordinator(req),
            KafkaPacket::JoinGroupReq(req) => consumer_group::process_join_group(req),
            KafkaPacket::HeartbeatReq(req) => consumer_group::process_heartbeat(req),
            KafkaPacket::LeaveGroupReq(req) => consumer_group::process_leave_group(req),
            KafkaPacket::SyncGroupReq(req) => consumer_group::process_sync_group(req),
            KafkaPacket::DescribeGroupsReq(req) => consumer_group::process_describe_groups(req),
            KafkaPacket::ListGroupsReq(req) => consumer_group::process_list_groups(req),
            KafkaPacket::DeleteGroupsReq(req) => consumer_group::process_delete_groups(req),
            KafkaPacket::OffsetDeleteReq(req) => consumer_group::process_offset_delete(req),
            // Connection & Authentication
            KafkaPacket::SaslHandshakeReq(req) => auth::process_sasl_handshake(req),
            KafkaPacket::ApiVersionReq(_) => self.process_api_versions(),
            KafkaPacket::SaslAuthenticateReq(req) => auth::process_sasl_authenticate(req),
            // Topic / Partition Management
            KafkaPacket::CreateTopicsReq(req) => topic::process_create_topics(req),
            KafkaPacket::DeleteTopicsReq(req) => topic::process_delete_topics(req),
            KafkaPacket::DeleteRecordsReq(req) => topic::process_delete_records(req),
            KafkaPacket::CreatePartitionsReq(req) => topic::process_create_partitions(req),
            // Configuration Management
            KafkaPacket::DescribeConfigsReq(req) => config::process_describe_configs(req),
            KafkaPacket::AlterConfigsReq(req) => config::process_alter_configs(req),
            KafkaPacket::IncrementalAlterConfigsReq(req) => {
                config::process_incremental_alter_configs(req)
            }
            // Transaction Support
            KafkaPacket::InitProducerIdReq(req) => transaction::process_init_producer_id(req),
            KafkaPacket::AddPartitionsToTxnReq(req) => {
                transaction::process_add_partitions_to_txn(req)
            }
            KafkaPacket::AddOffsetsToTxnReq(req) => transaction::process_add_offsets_to_txn(req),
            KafkaPacket::EndTxnReq(req) => transaction::process_end_txn(req),
            KafkaPacket::TxnOffsetCommitReq(req) => transaction::process_txn_offset_commit(req),
            KafkaPacket::DescribeTransactionsReq(req) => {
                transaction::process_describe_transactions(req)
            }
            KafkaPacket::ListTransactionsReq(req) => transaction::process_list_transactions(req),
            // ACL Access Control
            KafkaPacket::DescribeAclsReq(req) => acl::process_describe_acls(req),
            KafkaPacket::CreateAclsReq(req) => acl::process_create_acls(req),
            KafkaPacket::DeleteAclsReq(req) => acl::process_delete_acls(req),
            // Quota Management
            KafkaPacket::DescribeClientQuotasReq(req) => quota::process_describe_client_quotas(req),
            KafkaPacket::AlterClientQuotasReq(req) => quota::process_alter_client_quotas(req),
            KafkaPacket::DescribeUserScramCredentialsReq(req) => {
                quota::process_describe_user_scram_credentials(req)
            }
            KafkaPacket::AlterUserScramCredentialsReq(req) => {
                quota::process_alter_user_scram_credentials(req)
            }
            // Delegation Token Authentication
            KafkaPacket::CreateDelegationTokenReq(req) => {
                delegation_token::process_create_delegation_token(req)
            }
            KafkaPacket::RenewDelegationTokenReq(req) => {
                delegation_token::process_renew_delegation_token(req)
            }
            KafkaPacket::ExpireDelegationTokenReq(req) => {
                delegation_token::process_expire_delegation_token(req)
            }
            KafkaPacket::DescribeDelegationTokenReq(req) => {
                delegation_token::process_describe_delegation_token(req)
            }
            // Client Telemetry
            KafkaPacket::GetTelemetrySubscriptionsReq(req) => {
                telemetry::process_get_telemetry_subscriptions(req)
            }
            KafkaPacket::PushTelemetryReq(req) => telemetry::process_push_telemetry(req),
            KafkaPacket::ListConfigResourcesReq(req) => {
                telemetry::process_list_config_resources(req)
            }
            // Operations & Administration
            KafkaPacket::OffsetForLeaderEpochReq(req) => {
                admin::process_offset_for_leader_epoch(req)
            }
            KafkaPacket::AlterReplicaLogDirsReq(req) => admin::process_alter_replica_log_dirs(req),
            KafkaPacket::DescribeLogDirsReq(req) => admin::process_describe_log_dirs(req),
            KafkaPacket::ElectLeadersReq(req) => admin::process_elect_leaders(req),
            KafkaPacket::AlterPartitionReassignmentsReq(req) => {
                admin::process_alter_partition_reassignments(req)
            }
            KafkaPacket::ListPartitionReassignmentsReq(req) => {
                admin::process_list_partition_reassignments(req)
            }
            KafkaPacket::UpdateFeaturesReq(req) => admin::process_update_features(req),
            KafkaPacket::DescribeClusterReq(req) => admin::process_describe_cluster(req),
            KafkaPacket::DescribeProducersReq(req) => admin::process_describe_producers(req),
            KafkaPacket::DescribeTopicPartitionsReq(req) => {
                admin::process_describe_topic_partitions(req)
            }
            // Next-Generation Consumer Group Protocol (KIP-848)
            KafkaPacket::ConsumerGroupHeartbeatReq(req) => {
                consumer_group_next::process_consumer_group_heartbeat(req)
            }
            KafkaPacket::ConsumerGroupDescribeReq(req) => {
                consumer_group_next::process_consumer_group_describe(req)
            }
            // Share Group (KIP-932)
            KafkaPacket::ShareGroupHeartbeatReq(req) => {
                share_group::process_share_group_heartbeat(req)
            }
            KafkaPacket::ShareGroupDescribeReq(req) => {
                share_group::process_share_group_describe(req)
            }
            KafkaPacket::ShareFetchReq(req) => share_group::process_share_fetch(req),
            KafkaPacket::ShareAcknowledgeReq(req) => share_group::process_share_acknowledge(req),
            KafkaPacket::DescribeShareGroupOffsetsReq(req) => {
                share_group::process_describe_share_group_offsets(req)
            }
            KafkaPacket::AlterShareGroupOffsetsReq(req) => {
                share_group::process_alter_share_group_offsets(req)
            }
            KafkaPacket::DeleteShareGroupOffsetsReq(req) => {
                share_group::process_delete_share_group_offsets(req)
            }
            // Response variants — not handled by server
            other => {
                warn!(
                    connection_id,
                    api_key = ?other,
                    "Received unexpected response packet from client"
                );
                return None;
            }
        }?;

        let resp_header = ResponseHeader::default().with_correlation_id(correlation_id);
        let resp_wrapper = KafkaPacketWrapper {
            api_version: wrapper.api_version,
            header: KafkaHeader::Response(resp_header),
            packet: resp_packet,
        };

        Some(ResponsePackage::new(
            connection_id,
            RobustMQPacket::KAFKA(resp_wrapper),
        ))
    }
}

impl KafkaHandlerCommand {
    fn process_api_versions(&self) -> Option<KafkaPacket> {
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

    fn process_metadata(&self, req: &MetadataRequest) -> Option<KafkaPacket> {
        let mut topics = Vec::new();
        if let Some(req_topics) = &req.topics {
            for topic in req_topics {
                let topic_name = match &topic.name {
                    Some(n) => n.clone(),
                    None => continue,
                };
                topics.push(
                    MetadataResponseTopic::default()
                        .with_error_code(0)
                        .with_name(Some(topic_name))
                        .with_is_internal(false)
                        .with_partitions(vec![MetadataResponsePartition::default()
                            .with_error_code(0)
                            .with_partition_index(0)
                            .with_leader_id(0.into())
                            .with_replica_nodes(vec![0.into()])
                            .with_isr_nodes(vec![0.into()])]),
                );
            }
        }

        let broker = MetadataResponseBroker::default()
            .with_node_id(0.into())
            .with_host("127.0.0.1".into())
            .with_port(9095);

        let resp = MetadataResponse::default()
            .with_brokers(vec![broker])
            .with_controller_id(0.into())
            .with_topics(topics);

        Some(KafkaPacket::MetadataResponse(resp))
    }

    fn process_find_coordinator(&self, _req: &FindCoordinatorRequest) -> Option<KafkaPacket> {
        let resp = FindCoordinatorResponse::default()
            .with_error_code(0)
            .with_node_id(0.into())
            .with_host("127.0.0.1".into())
            .with_port(9095)
            .with_coordinators(vec![Coordinator::default()
                .with_error_code(0)
                .with_node_id(0.into())
                .with_host("127.0.0.1".into())
                .with_port(9095)]);

        Some(KafkaPacket::FindCoordinatorResponse(resp))
    }

    async fn process_fetch(&self, req: &FetchRequest, connection_id: u64) -> Option<KafkaPacket> {
        let sdm = self.storage_driver_manager.as_ref()?;

        let read_config = AdapterReadConfig::new();
        let mut topic_responses = Vec::new();

        for fetch_topic in &req.topics {
            let topic_name = fetch_topic.topic.to_string();
            let key = (connection_id, topic_name.clone());

            let mut offsets = self
                .shard_offsets
                .get(&key)
                .map(|r| r.clone())
                .unwrap_or_default();

            let records_bytes = match sdm
                .read_by_offset(DEFAULT_TENANT, &topic_name, &offsets, &read_config)
                .await
            {
                Ok(records) if records.is_empty() => None,
                Ok(records) => {
                    let mut kafka_records = Vec::new();
                    for (i, record) in records.iter().enumerate() {
                        offsets.insert(record.metadata.shard.clone(), record.metadata.offset + 1);
                        kafka_records.push(Record {
                            transactional: false,
                            control: false,
                            partition_leader_epoch: 0,
                            producer_id: -1,
                            producer_epoch: -1,
                            timestamp_type: TimestampType::Creation,
                            offset: record.metadata.offset as i64,
                            sequence: i as i32,
                            timestamp: 0,
                            key: None,
                            value: Some(record.data.clone()),
                            headers: Default::default(),
                        });
                    }
                    self.shard_offsets.insert(key, offsets);

                    let mut buf = bytes::BytesMut::new();
                    let opts = RecordEncodeOptions {
                        version: 2,
                        compression: Compression::None,
                    };
                    RecordBatchEncoder::encode(&mut buf, kafka_records.iter(), &opts).ok()?;
                    Some(buf.freeze())
                }
                Err(e) => {
                    warn!("Kafka Fetch storage error for {}: {}", topic_name, e);
                    None
                }
            };

            let mut partition_responses = Vec::new();
            for fetch_partition in &fetch_topic.partitions {
                partition_responses.push(
                    PartitionData::default()
                        .with_partition_index(fetch_partition.partition)
                        .with_error_code(0)
                        .with_high_watermark(i64::MAX)
                        .with_last_stable_offset(-1)
                        .with_log_start_offset(0)
                        .with_records(records_bytes.clone()),
                );
            }

            topic_responses.push(
                FetchableTopicResponse::default()
                    .with_topic(fetch_topic.topic.clone())
                    .with_partitions(partition_responses),
            );
        }

        let resp = FetchResponse::default()
            .with_error_code(0)
            .with_session_id(0)
            .with_responses(topic_responses);

        Some(KafkaPacket::FetchResponse(resp))
    }
}

pub fn create_command() -> Arc<Box<dyn Command + Send + Sync>> {
    Arc::new(Box::new(KafkaHandlerCommand::new()))
}

pub fn create_command_with_storage(
    storage_driver_manager: Arc<StorageDriverManager>,
) -> Arc<Box<dyn Command + Send + Sync>> {
    Arc::new(Box::new(KafkaHandlerCommand::new_with_storage(
        storage_driver_manager,
    )))
}
