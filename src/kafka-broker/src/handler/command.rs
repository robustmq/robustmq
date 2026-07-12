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

use std::sync::Arc;

use async_trait::async_trait;
use broker_core::cache::NodeCacheManager;
use common_config::broker::broker_config;
use kafka_protocol::messages::ResponseHeader;
use metadata_struct::connection::NetworkConnection;
use network_server::command::Command;
use network_server::common::packet::ResponsePackage;
use protocol::kafka::packet::{KafkaHeader, KafkaPacket, KafkaPacketWrapper};
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

use crate::core::cache::KafkaCacheManager;
use crate::core::coordinator::GroupCoordinator;
use crate::kafka::{
    acl, admin, api_versions, auth, config, consumer_group, consumer_group_next,
    consumer_group_offset, delegation_token, fetch, metadata, offset, produce, quota, scram,
    share_group, telemetry, topic, transaction,
};

#[derive(Clone)]
pub struct KafkaHandlerCommand {
    storage_driver_manager: Arc<StorageDriverManager>,
    broker_cache: Arc<NodeCacheManager>,
    kafka_cache: Arc<KafkaCacheManager>,
    group_coordinator: Arc<GroupCoordinator>,
}

impl KafkaHandlerCommand {
    pub fn new(
        storage_driver_manager: Arc<StorageDriverManager>,
        broker_cache: Arc<NodeCacheManager>,
        kafka_cache: Arc<KafkaCacheManager>,
    ) -> Self {
        KafkaHandlerCommand {
            storage_driver_manager,
            broker_cache,
            kafka_cache: kafka_cache.clone(),
            group_coordinator: Arc::new(GroupCoordinator::new(kafka_cache)),
        }
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

        // When SASL is enabled, an unauthenticated connection may only negotiate
        // versions or run the SASL handshake; any other request is dropped until
        // the connection authenticates.
        let sasl = &broker_config().kafka_runtime.sasl;
        if sasl.enabled
            && !self.kafka_cache.is_sasl_authenticated(connection_id)
            && !is_preauth_allowed(&wrapper.packet)
        {
            warn!(
                "Kafka request rejected on connection {}: SASL authentication required",
                connection_id
            );
            return None;
        }

        let resp_packet = match &wrapper.packet {
            // Core Data Plane
            KafkaPacket::ProduceReq(req) => {
                produce::process_produce(&self.storage_driver_manager, &self.kafka_cache, req).await
            }
            KafkaPacket::FetchReq(req) => {
                fetch::process_fetch(&self.storage_driver_manager, req).await
            }
            KafkaPacket::ListOffsetsReq(req) => {
                offset::process_list_offsets(&self.storage_driver_manager, req).await
            }
            KafkaPacket::MetadataReq(req) => {
                metadata::process_metadata(&self.broker_cache, &self.storage_driver_manager, req)
                    .await
            }
            // Consumer Group Management
            KafkaPacket::OffsetCommitReq(req) => {
                consumer_group_offset::process_offset_commit(&self.storage_driver_manager, req)
                    .await
            }
            KafkaPacket::OffsetFetchReq(req) => {
                consumer_group_offset::process_offset_fetch(&self.storage_driver_manager, req).await
            }
            KafkaPacket::FindCoordinatorReq(req) => {
                consumer_group::process_find_coordinator(
                    &self.storage_driver_manager,
                    wrapper.api_version,
                    req,
                )
                .await
            }
            KafkaPacket::JoinGroupReq(req) => {
                let client_id = match &wrapper.header {
                    KafkaHeader::Request(h) => h
                        .client_id
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    KafkaHeader::Response(_) => String::new(),
                };
                consumer_group::process_join_group(
                    &self.group_coordinator,
                    &self.storage_driver_manager,
                    wrapper.api_version,
                    client_id,
                    req,
                )
                .await
            }
            KafkaPacket::HeartbeatReq(req) => {
                consumer_group::process_heartbeat(
                    &self.group_coordinator,
                    &self.storage_driver_manager,
                    req,
                )
                .await
            }
            KafkaPacket::LeaveGroupReq(req) => {
                consumer_group::process_leave_group(
                    &self.group_coordinator,
                    &self.storage_driver_manager,
                    req,
                )
                .await
            }
            KafkaPacket::SyncGroupReq(req) => {
                consumer_group::process_sync_group(
                    &self.group_coordinator,
                    &self.storage_driver_manager,
                    req,
                )
                .await
            }
            KafkaPacket::DescribeGroupsReq(req) => {
                consumer_group::process_describe_groups(
                    &self.group_coordinator,
                    &self.storage_driver_manager,
                    req,
                )
                .await
            }
            KafkaPacket::ListGroupsReq(req) => {
                consumer_group::process_list_groups(&self.group_coordinator, req)
            }
            KafkaPacket::DeleteGroupsReq(req) => {
                consumer_group::process_delete_groups(
                    &self.group_coordinator,
                    &self.storage_driver_manager,
                    req,
                )
                .await
            }
            KafkaPacket::OffsetDeleteReq(req) => {
                offset::process_offset_delete(&self.storage_driver_manager, req).await
            }
            // Connection & Authentication
            KafkaPacket::SaslHandshakeReq(req) => {
                auth::process_sasl_handshake(&self.kafka_cache, connection_id, req)
            }
            KafkaPacket::ApiVersionReq(_) => api_versions::process_api_versions(),
            KafkaPacket::SaslAuthenticateReq(req) => {
                auth::process_sasl_authenticate(&self.kafka_cache, connection_id, req)
            }
            // Topic / Partition Management
            KafkaPacket::CreateTopicsReq(req) => {
                topic::process_create_topics(&self.storage_driver_manager, req).await
            }
            KafkaPacket::DeleteTopicsReq(req) => {
                topic::process_delete_topics(&self.storage_driver_manager, req).await
            }
            KafkaPacket::DeleteRecordsReq(req) => {
                topic::process_delete_records(&self.storage_driver_manager, req).await
            }
            KafkaPacket::CreatePartitionsReq(req) => {
                topic::process_create_partitions(&self.storage_driver_manager, req).await
            }
            // Configuration Management
            KafkaPacket::DescribeConfigsReq(req) => {
                config::process_describe_configs(&self.storage_driver_manager, req).await
            }
            KafkaPacket::AlterConfigsReq(req) => {
                config::process_alter_configs(&self.storage_driver_manager, req).await
            }
            KafkaPacket::IncrementalAlterConfigsReq(req) => {
                config::process_incremental_alter_configs(&self.storage_driver_manager, req).await
            }
            // Transaction Support
            KafkaPacket::InitProducerIdReq(req) => {
                transaction::process_init_producer_id(&self.kafka_cache, req)
            }
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
            KafkaPacket::DescribeAclsReq(req) => {
                acl::process_describe_acls(&self.storage_driver_manager, req).await
            }
            KafkaPacket::CreateAclsReq(req) => {
                acl::process_create_acls(&self.storage_driver_manager, req).await
            }
            KafkaPacket::DeleteAclsReq(req) => {
                acl::process_delete_acls(&self.storage_driver_manager, req).await
            }
            // Quota Management
            KafkaPacket::DescribeClientQuotasReq(req) => {
                quota::process_describe_client_quotas(&self.storage_driver_manager, req).await
            }
            KafkaPacket::AlterClientQuotasReq(req) => {
                quota::process_alter_client_quotas(&self.storage_driver_manager, req).await
            }
            KafkaPacket::DescribeUserScramCredentialsReq(req) => {
                scram::process_describe_user_scram_credentials(&self.storage_driver_manager, req)
                    .await
            }
            KafkaPacket::AlterUserScramCredentialsReq(req) => {
                scram::process_alter_user_scram_credentials(&self.storage_driver_manager, req).await
            }
            // Delegation Token Authentication
            KafkaPacket::CreateDelegationTokenReq(req) => {
                delegation_token::process_create_delegation_token(&self.storage_driver_manager, req)
                    .await
            }
            KafkaPacket::RenewDelegationTokenReq(req) => {
                delegation_token::process_renew_delegation_token(&self.storage_driver_manager, req)
                    .await
            }
            KafkaPacket::ExpireDelegationTokenReq(req) => {
                delegation_token::process_expire_delegation_token(&self.storage_driver_manager, req)
                    .await
            }
            KafkaPacket::DescribeDelegationTokenReq(req) => {
                delegation_token::process_describe_delegation_token(
                    &self.storage_driver_manager,
                    req,
                )
                .await
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
            KafkaPacket::DescribeClusterReq(req) => {
                metadata::process_describe_cluster(
                    &self.broker_cache,
                    &self.storage_driver_manager,
                    req,
                )
                .await
            }
            KafkaPacket::DescribeProducersReq(req) => admin::process_describe_producers(req),
            KafkaPacket::DescribeTopicPartitionsReq(req) => {
                metadata::process_describe_topic_partitions(
                    &self.broker_cache,
                    &self.storage_driver_manager,
                    req,
                )
            }
            // Next-Generation Consumer Group Protocol (KIP-848)
            KafkaPacket::ConsumerGroupHeartbeatReq(req) => {
                let client_id = match &wrapper.header {
                    KafkaHeader::Request(h) => h
                        .client_id
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    KafkaHeader::Response(_) => String::new(),
                };
                consumer_group_next::process_consumer_group_heartbeat(
                    &self.group_coordinator,
                    &self.storage_driver_manager,
                    client_id,
                    req,
                )
                .await
            }
            KafkaPacket::ConsumerGroupDescribeReq(req) => {
                consumer_group_next::process_consumer_group_describe(
                    &self.group_coordinator,
                    &self.storage_driver_manager,
                    req,
                )
                .await
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

// Requests an unauthenticated connection may send while SASL is enabled.
fn is_preauth_allowed(packet: &KafkaPacket) -> bool {
    matches!(
        packet,
        KafkaPacket::ApiVersionReq(_)
            | KafkaPacket::SaslHandshakeReq(_)
            | KafkaPacket::SaslAuthenticateReq(_)
    )
}

pub fn create_command(
    storage_driver_manager: Arc<StorageDriverManager>,
    broker_cache: Arc<NodeCacheManager>,
    kafka_cache: Arc<KafkaCacheManager>,
) -> Arc<Box<dyn Command + Send + Sync>> {
    Arc::new(Box::new(KafkaHandlerCommand::new(
        storage_driver_manager,
        broker_cache,
        kafka_cache,
    )))
}
