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
use dashmap::DashMap;
use kafka_protocol::messages::ResponseHeader;
use metadata_struct::connection::NetworkConnection;
use network_server::command::Command;
use network_server::common::packet::ResponsePackage;
use protocol::kafka::packet::{KafkaHeader, KafkaPacket, KafkaPacketWrapper};
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

use crate::kafka::core::ShardOffsets;
use crate::kafka::{
    acl, admin, api_versions, auth, config, consumer_group, consumer_group_next, core,
    delegation_token, find_coordinator, metadata, quota, share_group, telemetry, topic,
    transaction,
};

#[derive(Clone)]
pub struct KafkaHandlerCommand {
    storage_driver_manager: Arc<StorageDriverManager>,
    broker_cache: Arc<NodeCacheManager>,
    // (connection_id, topic) -> per-shard offsets
    shard_offsets: ShardOffsets,
}

impl KafkaHandlerCommand {
    pub fn new(
        storage_driver_manager: Arc<StorageDriverManager>,
        broker_cache: Arc<NodeCacheManager>,
    ) -> Self {
        KafkaHandlerCommand {
            storage_driver_manager,
            broker_cache,
            shard_offsets: Arc::new(DashMap::new()),
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

        let resp_packet = match &wrapper.packet {
            // Core Data Plane
            KafkaPacket::ProduceReq(req) => {
                core::process_produce(&self.storage_driver_manager, req).await
            }
            KafkaPacket::FetchReq(req) => {
                core::process_fetch(
                    &self.storage_driver_manager,
                    &self.shard_offsets,
                    req,
                    connection_id,
                )
                .await
            }
            KafkaPacket::ListOffsetsReq(req) => {
                core::process_list_offsets(&self.storage_driver_manager, req).await
            }
            KafkaPacket::MetadataReq(req) => {
                metadata::process_metadata(&self.broker_cache, &self.storage_driver_manager, req)
            }
            // Consumer Group Management
            KafkaPacket::OffsetCommitReq(req) => {
                consumer_group::process_offset_commit(&self.storage_driver_manager, req).await
            }
            KafkaPacket::OffsetFetchReq(req) => {
                consumer_group::process_offset_fetch(&self.storage_driver_manager, req).await
            }
            KafkaPacket::FindCoordinatorReq(req) => find_coordinator::process_find_coordinator(req),
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
            KafkaPacket::ApiVersionReq(_) => api_versions::process_api_versions(),
            KafkaPacket::SaslAuthenticateReq(req) => auth::process_sasl_authenticate(req),
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

pub fn create_command(
    storage_driver_manager: Arc<StorageDriverManager>,
    broker_cache: Arc<NodeCacheManager>,
) -> Arc<Box<dyn Command + Send + Sync>> {
    Arc::new(Box::new(KafkaHandlerCommand::new(
        storage_driver_manager,
        broker_cache,
    )))
}
