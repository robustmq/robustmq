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

use crate::kafka::packet::{KafkaHeader, KafkaPacket, KafkaPacketWrapper};
use bytes::{Buf, BufMut, BytesMut};
use common_base::error::common::CommonError;
use kafka_protocol::{
    messages::{
        AddOffsetsToTxnRequest,
        AddPartitionsToTxnRequest,
        AlterClientQuotasRequest,
        AlterConfigsRequest,
        AlterPartitionReassignmentsRequest,
        AlterReplicaLogDirsRequest,
        AlterShareGroupOffsetsRequest,
        AlterUserScramCredentialsRequest,
        // ApiKey
        ApiKey,
        ApiVersionsRequest,
        ConsumerGroupDescribeRequest,
        // Next-Generation Consumer Group Protocol (KIP-848)
        ConsumerGroupHeartbeatRequest,
        CreateAclsRequest,
        // Delegation Token Authentication
        CreateDelegationTokenRequest,
        CreatePartitionsRequest,
        // Topic / Partition Management
        CreateTopicsRequest,
        DeleteAclsRequest,
        DeleteGroupsRequest,
        DeleteRecordsRequest,
        DeleteShareGroupOffsetsRequest,
        DeleteTopicsRequest,
        // ACL Access Control
        DescribeAclsRequest,
        // Quota Management
        DescribeClientQuotasRequest,
        DescribeClusterRequest,
        // Configuration Management
        DescribeConfigsRequest,
        DescribeDelegationTokenRequest,
        DescribeGroupsRequest,
        DescribeLogDirsRequest,
        DescribeProducersRequest,
        DescribeShareGroupOffsetsRequest,
        DescribeTopicPartitionsRequest,
        DescribeTransactionsRequest,
        DescribeUserScramCredentialsRequest,
        ElectLeadersRequest,
        EndTxnRequest,
        ExpireDelegationTokenRequest,
        FetchRequest,
        FindCoordinatorRequest,
        // Client Telemetry
        GetTelemetrySubscriptionsRequest,
        HeartbeatRequest,
        IncrementalAlterConfigsRequest,
        // Transaction Support
        InitProducerIdRequest,
        JoinGroupRequest,
        LeaveGroupRequest,
        ListConfigResourcesRequest,
        ListGroupsRequest,
        ListOffsetsRequest,
        ListPartitionReassignmentsRequest,
        ListTransactionsRequest,
        MetadataRequest,
        // Consumer Group Management
        OffsetCommitRequest,
        OffsetDeleteRequest,
        OffsetFetchRequest,
        // Operations & Administration
        OffsetForLeaderEpochRequest,
        // Core Data Plane
        ProduceRequest,
        PushTelemetryRequest,
        RenewDelegationTokenRequest,
        RequestHeader,
        SaslAuthenticateRequest,
        // Connection & Authentication
        SaslHandshakeRequest,
        ShareAcknowledgeRequest,
        ShareFetchRequest,
        ShareGroupDescribeRequest,
        // Share Group (KIP-932)
        ShareGroupHeartbeatRequest,
        SyncGroupRequest,
        TxnOffsetCommitRequest,
        UpdateFeaturesRequest,
    },
    protocol::{Decodable, Encodable},
};
use std::convert::TryFrom;
use std::io::{Cursor, Error, ErrorKind};
use tokio_util::codec;

#[derive(Clone, Debug)]
pub struct KafkaCodec {}

impl KafkaCodec {
    pub fn new() -> KafkaCodec {
        KafkaCodec {}
    }
}

impl Default for KafkaCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl KafkaCodec {
    pub fn decode_data(
        &mut self,
        stream: &mut BytesMut,
    ) -> Result<Option<KafkaPacketWrapper>, CommonError> {
        if stream.len() < 4 {
            return Ok(None);
        }

        let total_len = (&stream[..4]).get_i32() as usize;
        if stream.len() < 4 + total_len {
            return Ok(None);
        }

        stream.advance(4);

        let mut buf = Cursor::new(stream.split_to(total_len));

        // Peek at api_key and api_version to determine header version dynamically.
        // RequestHeader fields: request_api_key (i16), request_api_version (i16), ...
        // We need to read with the correct header version, so first peek the two i16 values.
        let buf_position = buf.position();
        if (buf.get_ref().len() as u64 - buf_position) < 4 {
            return Err(CommonError::CommonError(
                "Kafka frame too short to read api_key and api_version".to_string(),
            ));
        }
        let api_key_raw = (&buf.get_ref()[(buf_position as usize)..]).get_i16();
        let api_version = (&buf.get_ref()[(buf_position as usize + 2)..]).get_i16();

        let header_version = ApiKey::try_from(api_key_raw)
            .map(|k| k.request_header_version(api_version))
            .unwrap_or(2);

        // Reset and decode header with correct version
        buf.set_position(buf_position);
        let header = RequestHeader::decode(&mut buf, header_version).map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Header decode failed: {e}"))
        })?;

        let req = match header.request_api_key {
            // 0: Produce
            0 => KafkaPacket::ProduceReq(ProduceRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 1: Fetch
            1 => KafkaPacket::FetchReq(FetchRequest::decode(&mut buf, header.request_api_version)?),
            // 2: ListOffsets
            2 => KafkaPacket::ListOffsetsReq(ListOffsetsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 3: Metadata
            3 => KafkaPacket::MetadataReq(MetadataRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 8: OffsetCommit
            8 => KafkaPacket::OffsetCommitReq(OffsetCommitRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 9: OffsetFetch
            9 => KafkaPacket::OffsetFetchReq(OffsetFetchRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 10: FindCoordinator
            10 => KafkaPacket::FindCoordinatorReq(FindCoordinatorRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 11: JoinGroup
            11 => KafkaPacket::JoinGroupReq(JoinGroupRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 12: Heartbeat
            12 => KafkaPacket::HeartbeatReq(HeartbeatRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 13: LeaveGroup
            13 => KafkaPacket::LeaveGroupReq(LeaveGroupRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 14: SyncGroup
            14 => KafkaPacket::SyncGroupReq(SyncGroupRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 15: DescribeGroups
            15 => KafkaPacket::DescribeGroupsReq(DescribeGroupsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 16: ListGroups
            16 => KafkaPacket::ListGroupsReq(ListGroupsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 17: SaslHandshake
            17 => KafkaPacket::SaslHandshakeReq(SaslHandshakeRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 18: ApiVersions
            18 => KafkaPacket::ApiVersionReq(ApiVersionsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 19: CreateTopics
            19 => KafkaPacket::CreateTopicsReq(CreateTopicsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 20: DeleteTopics
            20 => KafkaPacket::DeleteTopicsReq(DeleteTopicsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 21: DeleteRecords
            21 => KafkaPacket::DeleteRecordsReq(DeleteRecordsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 22: InitProducerId
            22 => KafkaPacket::InitProducerIdReq(InitProducerIdRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 23: OffsetForLeaderEpoch
            23 => KafkaPacket::OffsetForLeaderEpochReq(OffsetForLeaderEpochRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 24: AddPartitionsToTxn
            24 => KafkaPacket::AddPartitionsToTxnReq(AddPartitionsToTxnRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 25: AddOffsetsToTxn
            25 => KafkaPacket::AddOffsetsToTxnReq(AddOffsetsToTxnRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 26: EndTxn
            26 => {
                KafkaPacket::EndTxnReq(EndTxnRequest::decode(&mut buf, header.request_api_version)?)
            }
            // 28: TxnOffsetCommit
            28 => KafkaPacket::TxnOffsetCommitReq(TxnOffsetCommitRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 29: DescribeAcls
            29 => KafkaPacket::DescribeAclsReq(DescribeAclsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 30: CreateAcls
            30 => KafkaPacket::CreateAclsReq(CreateAclsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 31: DeleteAcls
            31 => KafkaPacket::DeleteAclsReq(DeleteAclsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 32: DescribeConfigs
            32 => KafkaPacket::DescribeConfigsReq(DescribeConfigsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 33: AlterConfigs
            33 => KafkaPacket::AlterConfigsReq(AlterConfigsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 34: AlterReplicaLogDirs
            34 => KafkaPacket::AlterReplicaLogDirsReq(AlterReplicaLogDirsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 35: DescribeLogDirs
            35 => KafkaPacket::DescribeLogDirsReq(DescribeLogDirsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 36: SaslAuthenticate
            36 => KafkaPacket::SaslAuthenticateReq(SaslAuthenticateRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 37: CreatePartitions
            37 => KafkaPacket::CreatePartitionsReq(CreatePartitionsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 38: CreateDelegationToken
            38 => KafkaPacket::CreateDelegationTokenReq(CreateDelegationTokenRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 39: RenewDelegationToken
            39 => KafkaPacket::RenewDelegationTokenReq(RenewDelegationTokenRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 40: ExpireDelegationToken
            40 => KafkaPacket::ExpireDelegationTokenReq(ExpireDelegationTokenRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 41: DescribeDelegationToken
            41 => KafkaPacket::DescribeDelegationTokenReq(DescribeDelegationTokenRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 42: DeleteGroups
            42 => KafkaPacket::DeleteGroupsReq(DeleteGroupsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 43: ElectLeaders
            43 => KafkaPacket::ElectLeadersReq(ElectLeadersRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 44: IncrementalAlterConfigs
            44 => KafkaPacket::IncrementalAlterConfigsReq(IncrementalAlterConfigsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 45: AlterPartitionReassignments
            45 => KafkaPacket::AlterPartitionReassignmentsReq(
                AlterPartitionReassignmentsRequest::decode(&mut buf, header.request_api_version)?,
            ),
            // 46: ListPartitionReassignments
            46 => KafkaPacket::ListPartitionReassignmentsReq(
                ListPartitionReassignmentsRequest::decode(&mut buf, header.request_api_version)?,
            ),
            // 47: OffsetDelete
            47 => KafkaPacket::OffsetDeleteReq(OffsetDeleteRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 48: DescribeClientQuotas
            48 => KafkaPacket::DescribeClientQuotasReq(DescribeClientQuotasRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 49: AlterClientQuotas
            49 => KafkaPacket::AlterClientQuotasReq(AlterClientQuotasRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 50: DescribeUserScramCredentials
            50 => KafkaPacket::DescribeUserScramCredentialsReq(
                DescribeUserScramCredentialsRequest::decode(&mut buf, header.request_api_version)?,
            ),
            // 51: AlterUserScramCredentials
            51 => KafkaPacket::AlterUserScramCredentialsReq(
                AlterUserScramCredentialsRequest::decode(&mut buf, header.request_api_version)?,
            ),
            // 57: UpdateFeatures
            57 => KafkaPacket::UpdateFeaturesReq(UpdateFeaturesRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 60: DescribeCluster
            60 => KafkaPacket::DescribeClusterReq(DescribeClusterRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 61: DescribeProducers
            61 => KafkaPacket::DescribeProducersReq(DescribeProducersRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 65: DescribeTransactions
            65 => KafkaPacket::DescribeTransactionsReq(DescribeTransactionsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 66: ListTransactions
            66 => KafkaPacket::ListTransactionsReq(ListTransactionsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 68: ConsumerGroupHeartbeat (KIP-848)
            68 => KafkaPacket::ConsumerGroupHeartbeatReq(ConsumerGroupHeartbeatRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 69: ConsumerGroupDescribe (KIP-848)
            69 => KafkaPacket::ConsumerGroupDescribeReq(ConsumerGroupDescribeRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 71: GetTelemetrySubscriptions
            71 => KafkaPacket::GetTelemetrySubscriptionsReq(
                GetTelemetrySubscriptionsRequest::decode(&mut buf, header.request_api_version)?,
            ),
            // 72: PushTelemetry
            72 => KafkaPacket::PushTelemetryReq(PushTelemetryRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 74: ListConfigResources
            74 => KafkaPacket::ListConfigResourcesReq(ListConfigResourcesRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 75: DescribeTopicPartitions
            75 => KafkaPacket::DescribeTopicPartitionsReq(DescribeTopicPartitionsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 76: ShareGroupHeartbeat (KIP-932)
            76 => KafkaPacket::ShareGroupHeartbeatReq(ShareGroupHeartbeatRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 77: ShareGroupDescribe (KIP-932)
            77 => KafkaPacket::ShareGroupDescribeReq(ShareGroupDescribeRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 78: ShareFetch (KIP-932)
            78 => KafkaPacket::ShareFetchReq(ShareFetchRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 79: ShareAcknowledge (KIP-932)
            79 => KafkaPacket::ShareAcknowledgeReq(ShareAcknowledgeRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 90: DescribeShareGroupOffsets (KIP-932)
            90 => KafkaPacket::DescribeShareGroupOffsetsReq(
                DescribeShareGroupOffsetsRequest::decode(&mut buf, header.request_api_version)?,
            ),
            // 91: AlterShareGroupOffsets (KIP-932)
            91 => KafkaPacket::AlterShareGroupOffsetsReq(AlterShareGroupOffsetsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            // 92: DeleteShareGroupOffsets (KIP-932)
            92 => KafkaPacket::DeleteShareGroupOffsetsReq(DeleteShareGroupOffsetsRequest::decode(
                &mut buf,
                header.request_api_version,
            )?),
            _ => {
                return Err(CommonError::NotSupportKafkaRequest(header.request_api_key));
            }
        };

        Ok(Some(KafkaPacketWrapper {
            api_version: header.request_api_version,
            header: KafkaHeader::Request(header),
            packet: req,
        }))
    }

    pub fn encode_data(
        &mut self,
        wrapper: KafkaPacketWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), CommonError> {
        let mut header_bytes = BytesMut::new();
        let mut body_bytes = BytesMut::new();

        match wrapper.header {
            KafkaHeader::Request(header) => {
                let header_version = ApiKey::try_from(header.request_api_key)
                    .map(|k| k.request_header_version(header.request_api_version))
                    .unwrap_or(2);
                header.encode(&mut header_bytes, header_version)?;
                match wrapper.packet {
                    KafkaPacket::ProduceReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::FetchReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListOffsetsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::MetadataReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::OffsetCommitReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::OffsetFetchReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::FindCoordinatorReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::JoinGroupReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::HeartbeatReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::LeaveGroupReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::SyncGroupReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeGroupsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListGroupsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteGroupsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::OffsetDeleteReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::SaslHandshakeReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ApiVersionReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::SaslAuthenticateReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::CreateTopicsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteTopicsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteRecordsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::CreatePartitionsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeConfigsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterConfigsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::IncrementalAlterConfigsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::InitProducerIdReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AddPartitionsToTxnReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AddOffsetsToTxnReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::EndTxnReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::TxnOffsetCommitReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeTransactionsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListTransactionsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeAclsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::CreateAclsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteAclsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeClientQuotasReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterClientQuotasReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeUserScramCredentialsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterUserScramCredentialsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::CreateDelegationTokenReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::RenewDelegationTokenReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ExpireDelegationTokenReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeDelegationTokenReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::GetTelemetrySubscriptionsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::PushTelemetryReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListConfigResourcesReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::OffsetForLeaderEpochReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterReplicaLogDirsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeLogDirsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ElectLeadersReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterPartitionReassignmentsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListPartitionReassignmentsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::UpdateFeaturesReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeClusterReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeProducersReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeTopicPartitionsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ConsumerGroupHeartbeatReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ConsumerGroupDescribeReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ShareGroupHeartbeatReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ShareGroupDescribeReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ShareFetchReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ShareAcknowledgeReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeShareGroupOffsetsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterShareGroupOffsetsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteShareGroupOffsetsReq(req) => {
                        req.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    _ => {
                        return Err(CommonError::NotSupportKafkaEncodePacket(format!(
                            "{:?}",
                            wrapper.packet
                        )));
                    }
                }
            }
            KafkaHeader::Response(header) => {
                header.encode(&mut header_bytes, 1)?;
                match wrapper.packet {
                    KafkaPacket::ProduceResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::FetchResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListOffsetsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::MetadataResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::OffsetCommitResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::OffsetFetchResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::FindCoordinatorResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::JoinGroupResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::HeartbeatResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::LeaveGroupResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::SyncGroupResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeGroupsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListGroupsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteGroupsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::OffsetDeleteResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::SaslHandshakeResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ApiVersionResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::SaslAuthenticateResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::CreateTopicsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteTopicsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteRecordsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::CreatePartitionsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeConfigsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterConfigsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::IncrementalAlterConfigsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::InitProducerIdResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AddPartitionsToTxnResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AddOffsetsToTxnResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::EndTxnResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::TxnOffsetCommitResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeTransactionsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListTransactionsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeAclsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::CreateAclsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteAclsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeClientQuotasResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterClientQuotasResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeUserScramCredentialsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterUserScramCredentialsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::CreateDelegationTokenResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::RenewDelegationTokenResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ExpireDelegationTokenResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeDelegationTokenResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::GetTelemetrySubscriptionsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::PushTelemetryResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListConfigResourcesResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::OffsetForLeaderEpochResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterReplicaLogDirsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeLogDirsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ElectLeadersResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterPartitionReassignmentsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ListPartitionReassignmentsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::UpdateFeaturesResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeClusterResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeProducersResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeTopicPartitionsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ConsumerGroupHeartbeatResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ConsumerGroupDescribeResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ShareGroupHeartbeatResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ShareGroupDescribeResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ShareFetchResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::ShareAcknowledgeResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DescribeShareGroupOffsetsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::AlterShareGroupOffsetsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    KafkaPacket::DeleteShareGroupOffsetsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?
                    }
                    _ => {
                        return Err(CommonError::NotSupportKafkaEncodePacket(format!(
                            "{:?}",
                            wrapper.packet
                        )));
                    }
                }
            }
        }

        let total_len = header_bytes.len() + body_bytes.len();
        let len_byte = (total_len as u32).to_be_bytes();
        buffer.put_slice(&len_byte);
        buffer.put_slice(&header_bytes);
        buffer.put_slice(&body_bytes);

        Ok(())
    }
}

impl codec::Encoder<KafkaPacketWrapper> for KafkaCodec {
    type Error = CommonError;
    fn encode(
        &mut self,
        packet_wrapper: KafkaPacketWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        self.encode_data(packet_wrapper, buffer)
    }
}

impl codec::Decoder for KafkaCodec {
    type Item = KafkaPacketWrapper;
    type Error = CommonError;
    fn decode(&mut self, stream: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode_data(stream)
    }
}

#[cfg(test)]
mod tests {
    use crate::kafka::{
        codec::KafkaCodec,
        packet::{KafkaHeader, KafkaPacket, KafkaPacketWrapper},
    };
    use bytes::BytesMut;
    use kafka_protocol::{
        messages::{
            ApiKey, ApiVersionsRequest, CreateTopicsRequest, DeleteTopicsRequest,
            FindCoordinatorRequest, GroupId, HeartbeatRequest, JoinGroupRequest, LeaveGroupRequest,
            ListGroupsRequest, OffsetCommitRequest, OffsetFetchRequest, ProduceRequest,
            RequestHeader, SaslAuthenticateRequest, SaslHandshakeRequest, SyncGroupRequest,
        },
        protocol::StrBytes,
    };

    #[tokio::test]
    async fn producer_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        // ProduceRequest supports versions 3-13
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("my-client")))
            .with_request_api_key(ApiKey::Produce as i16)
            .with_request_api_version(9);

        let packet = ProduceRequest::default().with_acks(1).with_timeout_ms(3000);
        let wrapper = KafkaPacketWrapper {
            api_version: 9,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::ProduceReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        println!("{wrap:?}");
    }

    #[tokio::test]
    async fn api_versions_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::ApiVersions as i16)
            .with_request_api_version(3);

        let packet = ApiVersionsRequest::default()
            .with_client_software_name(StrBytes::from_static_str("my-app"))
            .with_client_software_version(StrBytes::from_static_str("1.0.0"));

        let wrapper = KafkaPacketWrapper {
            api_version: 3,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::ApiVersionReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::ApiVersionReq(req) => {
                    assert_eq!(
                        req.client_software_name,
                        StrBytes::from_static_str("my-app")
                    );
                }
                _ => panic!("Expected ApiVersionReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn sasl_authenticate_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::SaslAuthenticate as i16)
            .with_request_api_version(2);

        let packet = SaslAuthenticateRequest::default()
            .with_auth_bytes(bytes::Bytes::from_static(b"\x00user\x00pass"));

        let wrapper = KafkaPacketWrapper {
            api_version: 2,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::SaslAuthenticateReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::SaslAuthenticateReq(req) => {
                    assert_eq!(
                        req.auth_bytes,
                        bytes::Bytes::from_static(b"\x00user\x00pass")
                    );
                }
                _ => panic!("Expected SaslAuthenticateReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn offset_commit_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::OffsetCommit as i16)
            .with_request_api_version(3);

        let packet = OffsetCommitRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")));

        let wrapper = KafkaPacketWrapper {
            api_version: 3,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::OffsetCommitReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::OffsetCommitReq(req) => {
                    assert_eq!(
                        req.group_id,
                        GroupId(StrBytes::from_static_str("test-group"))
                    );
                }
                _ => panic!("Expected OffsetCommitReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn offset_fetch_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::OffsetFetch as i16)
            .with_request_api_version(6);

        let packet = OffsetFetchRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")));

        let wrapper = KafkaPacketWrapper {
            api_version: 6,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::OffsetFetchReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::OffsetFetchReq(req) => {
                    assert_eq!(
                        req.group_id,
                        GroupId(StrBytes::from_static_str("test-group"))
                    );
                }
                _ => panic!("Expected OffsetFetchReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn find_coordinator_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::FindCoordinator as i16)
            .with_request_api_version(3);

        let packet =
            FindCoordinatorRequest::default().with_key(StrBytes::from_static_str("test-group"));

        let wrapper = KafkaPacketWrapper {
            api_version: 3,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::FindCoordinatorReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::FindCoordinatorReq(req) => {
                    assert_eq!(req.key, StrBytes::from_static_str("test-group"));
                }
                _ => panic!("Expected FindCoordinatorReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn join_group_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::JoinGroup as i16)
            .with_request_api_version(7);

        let packet = JoinGroupRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")))
            .with_member_id(StrBytes::from_static_str("test-member"));

        let wrapper = KafkaPacketWrapper {
            api_version: 7,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::JoinGroupReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::JoinGroupReq(req) => {
                    assert_eq!(
                        req.group_id,
                        GroupId(StrBytes::from_static_str("test-group"))
                    );
                    assert_eq!(req.member_id, StrBytes::from_static_str("test-member"));
                }
                _ => panic!("Expected JoinGroupReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn heartbeat_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::Heartbeat as i16)
            .with_request_api_version(4);

        let packet = HeartbeatRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")))
            .with_member_id(StrBytes::from_static_str("test-member"));

        let wrapper = KafkaPacketWrapper {
            api_version: 4,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::HeartbeatReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::HeartbeatReq(req) => {
                    assert_eq!(
                        req.group_id,
                        GroupId(StrBytes::from_static_str("test-group"))
                    );
                    assert_eq!(req.member_id, StrBytes::from_static_str("test-member"));
                }
                _ => panic!("Expected HeartbeatReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn leave_group_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::LeaveGroup as i16)
            .with_request_api_version(0);

        let packet = LeaveGroupRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")));

        let wrapper = KafkaPacketWrapper {
            api_version: 0,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::LeaveGroupReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::LeaveGroupReq(req) => {
                    assert_eq!(
                        req.group_id,
                        GroupId(StrBytes::from_static_str("test-group"))
                    );
                }
                _ => panic!("Expected LeaveGroupReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn sync_group_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::SyncGroup as i16)
            .with_request_api_version(5);

        let packet = SyncGroupRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")))
            .with_member_id(StrBytes::from_static_str("test-member"));

        let wrapper = KafkaPacketWrapper {
            api_version: 5,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::SyncGroupReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::SyncGroupReq(req) => {
                    assert_eq!(
                        req.group_id,
                        GroupId(StrBytes::from_static_str("test-group"))
                    );
                    assert_eq!(req.member_id, StrBytes::from_static_str("test-member"));
                }
                _ => panic!("Expected SyncGroupReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn list_groups_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::ListGroups as i16)
            .with_request_api_version(4);

        let packet = ListGroupsRequest::default();

        let wrapper = KafkaPacketWrapper {
            api_version: 4,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::ListGroupsReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::ListGroupsReq(_) => {}
                _ => panic!("Expected ListGroupsReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn sasl_handshake_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::SaslHandshake as i16)
            .with_request_api_version(1);

        let packet =
            SaslHandshakeRequest::default().with_mechanism(StrBytes::from_static_str("PLAIN"));

        let wrapper = KafkaPacketWrapper {
            api_version: 1,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::SaslHandshakeReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::SaslHandshakeReq(req) => {
                    assert_eq!(req.mechanism, StrBytes::from_static_str("PLAIN"));
                }
                _ => panic!("Expected SaslHandshakeReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn create_topics_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::CreateTopics as i16)
            .with_request_api_version(7);

        let packet = CreateTopicsRequest::default();

        let wrapper = KafkaPacketWrapper {
            api_version: 7,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::CreateTopicsReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::CreateTopicsReq(_) => {}
                _ => panic!("Expected CreateTopicsReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn delete_topics_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::DeleteTopics as i16)
            .with_request_api_version(6);

        let packet = DeleteTopicsRequest::default();

        let wrapper = KafkaPacketWrapper {
            api_version: 6,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::DeleteTopicsReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());
        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::DeleteTopicsReq(_) => {}
                _ => panic!("Expected DeleteTopicsReq packet"),
            }
        }
    }
}
