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
use kafka_protocol::messages::ApiKey;
use kafka_protocol::{
    messages::{
        ApiVersionsRequest, CreateTopicsRequest, DeleteTopicsRequest, DescribeGroupsRequest,
        FetchRequest, FindCoordinatorRequest, HeartbeatRequest, JoinGroupRequest,
        LeaveGroupRequest, ListGroupsRequest, ListOffsetsRequest, MetadataRequest,
        OffsetCommitRequest, OffsetFetchRequest, ProduceRequest, RequestHeader,
        SaslHandshakeRequest, SyncGroupRequest,
    },
    protocol::{Decodable, Encodable},
};
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

        let header = RequestHeader::decode(&mut buf, 2).map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Header decode failed: {e}"))
        })?;

        let req = match header.request_api_key {
            0 => {
                let req = ProduceRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::ProduceReq(req)
            }
            1 => {
                let req = FetchRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::FetchReq(req)
            }

            2 => {
                let req = ListOffsetsRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::ListOffsetsReq(req)
            }

            3 => {
                let req = MetadataRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::MetadataReq(req)
            }

            8 => {
                let req = OffsetCommitRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::OffsetCommitReq(req)
            }

            9 => {
                let req = OffsetFetchRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::OffsetFetchReq(req)
            }

            10 => {
                let req = FindCoordinatorRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::FindCoordinatorReq(req)
            }

            11 => {
                let req = JoinGroupRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::JoinGroupReq(req)
            }

            12 => {
                let req = HeartbeatRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::HeartbeatReq(req)
            }

            13 => {
                let req = LeaveGroupRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::LeaveGroupReq(req)
            }

            14 => {
                let req = SyncGroupRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::SyncGroupReq(req)
            }

            15 => {
                let req = DescribeGroupsRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::DescribeGroupsReq(req)
            }

            16 => {
                let req = ListGroupsRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::ListGroupsReq(req)
            }

            17 => {
                let req = SaslHandshakeRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::SaslHandshakeReq(req)
            }

            18 => {
                let req = ApiVersionsRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::ApiVersionReq(req)
            }

            19 => {
                let req = CreateTopicsRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::CreateTopicsReq(req)
            }

            20 => {
                let req = DeleteTopicsRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::DeleteTopicsReq(req)
            }

            _ => {
                return Err(CommonError::NotSupportKafkaRequest(header.request_api_key));
            }
        };
        Ok(Some(KafkaPacketWrapper {
            api_key: header.request_api_key,
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
                header.encode(&mut header_bytes, 2)?;
                match wrapper.packet {
                    KafkaPacket::ProduceReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::MetadataReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::OffsetCommitReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::OffsetFetchReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::FindCoordinatorReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::JoinGroupReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::HeartbeatReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::LeaveGroupReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::SyncGroupReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::DescribeGroupsReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::ListGroupsReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::SaslHandshakeReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::CreateTopicsReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
                    }
                    KafkaPacket::DeleteTopicsReq(rep) => {
                        rep.encode(&mut body_bytes, header.request_api_version)?;
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
                // Determine the header version based on the API key
                let header_version = match ApiKey::try_from(wrapper.api_key) {
                    Ok(api_key_enum) => api_key_enum.response_header_version(wrapper.api_version),
                    Err(_) => 0, // Default to version 0 for unknown API keys
                };
                header.encode(&mut header_bytes, header_version)?;
                match wrapper.packet {
                    KafkaPacket::ProduceResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::FetchResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::ListOffsetsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::MetadataResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::OffsetCommitResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::OffsetFetchResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::FindCoordinatorResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::JoinGroupResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::HeartbeatResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::LeaveGroupResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::SyncGroupResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::DescribeGroupsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::ListGroupsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::SaslHandshakeResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::ApiVersionResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::CreateTopicsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
                    }
                    KafkaPacket::DeleteTopicsResponse(rep) => {
                        rep.encode(&mut body_bytes, wrapper.api_version)?;
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
            ApiKey, CreateTopicsRequest, DeleteTopicsRequest, FindCoordinatorRequest, GroupId,
            HeartbeatRequest, JoinGroupRequest, LeaveGroupRequest, ListGroupsRequest,
            OffsetCommitRequest, OffsetFetchRequest, ProduceRequest, RequestHeader,
            SaslHandshakeRequest, SyncGroupRequest,
        },
        protocol::StrBytes,
    };

    #[tokio::test]
    async fn producer_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        // encode
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("my-client")))
            .with_request_api_key(ApiKey::Produce as i16)
            .with_request_api_version(2);

        let packet = ProduceRequest::default().with_acks(1).with_timeout_ms(3000);
        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::Produce as i16,
            api_version: 2,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::ProduceReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
        let wrap = codec.decode_data(&mut buffer).unwrap();
        println!("{wrap:?}");
    }

    #[tokio::test]
    async fn offset_commit_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        // encode OffsetCommit request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::OffsetCommit as i16)
            .with_request_api_version(3);

        let packet = OffsetCommitRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")));

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::OffsetCommit as i16,
            api_version: 3,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::OffsetCommitReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
        let wrap = codec.decode_data(&mut buffer).unwrap();
        println!("OffsetCommit decode result: {wrap:?}");
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

        // encode OffsetFetch request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::OffsetFetch as i16)
            .with_request_api_version(6);

        let packet = OffsetFetchRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")));

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::OffsetFetch as i16,
            api_version: 6,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::OffsetFetchReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
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

        // encode FindCoordinator request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::FindCoordinator as i16)
            .with_request_api_version(3);

        let packet =
            FindCoordinatorRequest::default().with_key(StrBytes::from_static_str("test-group"));

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::FindCoordinator as i16,
            api_version: 3,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::FindCoordinatorReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
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

        // encode JoinGroup request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::JoinGroup as i16)
            .with_request_api_version(7);

        let packet = JoinGroupRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")))
            .with_member_id(StrBytes::from_static_str("test-member"));

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::JoinGroup as i16,
            api_version: 7,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::JoinGroupReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
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

        // encode Heartbeat request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::Heartbeat as i16)
            .with_request_api_version(4);

        let packet = HeartbeatRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")))
            .with_member_id(StrBytes::from_static_str("test-member"));

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::Heartbeat as i16,
            api_version: 4,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::HeartbeatReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
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

        // encode LeaveGroup request (use version 0 for basic support)
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::LeaveGroup as i16)
            .with_request_api_version(0);

        let packet = LeaveGroupRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")));

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::LeaveGroup as i16,
            api_version: 0,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::LeaveGroupReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
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

        // encode SyncGroup request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::SyncGroup as i16)
            .with_request_api_version(5);

        let packet = SyncGroupRequest::default()
            .with_group_id(GroupId(StrBytes::from_static_str("test-group")))
            .with_member_id(StrBytes::from_static_str("test-member"));

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::SyncGroup as i16,
            api_version: 5,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::SyncGroupReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
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

        // encode ListGroups request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::ListGroups as i16)
            .with_request_api_version(4);

        let packet = ListGroupsRequest::default();

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::ListGroups as i16,
            api_version: 4,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::ListGroupsReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());

        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::ListGroupsReq(_) => {
                    // ListGroups request has no specific fields to verify
                }
                _ => panic!("Expected ListGroupsReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn sasl_handshake_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        // encode SaslHandshake request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::SaslHandshake as i16)
            .with_request_api_version(1);

        let packet =
            SaslHandshakeRequest::default().with_mechanism(StrBytes::from_static_str("PLAIN"));

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::SaslHandshake as i16,
            api_version: 1,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::SaslHandshakeReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
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

        // encode CreateTopics request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::CreateTopics as i16)
            .with_request_api_version(7);

        let packet = CreateTopicsRequest::default();

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::CreateTopics as i16,
            api_version: 7,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::CreateTopicsReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());

        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::CreateTopicsReq(_) => {
                    // CreateTopics request verification
                }
                _ => panic!("Expected CreateTopicsReq packet"),
            }
        }
    }

    #[tokio::test]
    async fn delete_topics_req_test() {
        let mut codec = KafkaCodec::new();
        let mut buffer = BytesMut::new();

        // encode DeleteTopics request
        let header = RequestHeader::default()
            .with_client_id(Some(StrBytes::from_static_str("test-client")))
            .with_request_api_key(ApiKey::DeleteTopics as i16)
            .with_request_api_version(6);

        let packet = DeleteTopicsRequest::default();

        let wrapper = KafkaPacketWrapper {
            api_key: ApiKey::DeleteTopics as i16,
            api_version: 6,
            header: KafkaHeader::Request(header),
            packet: KafkaPacket::DeleteTopicsReq(packet),
        };
        codec.encode_data(wrapper, &mut buffer).unwrap();

        // decode
        let wrap = codec.decode_data(&mut buffer).unwrap();
        assert!(wrap.is_some());

        if let Some(wrapper) = wrap {
            match wrapper.packet {
                KafkaPacket::DeleteTopicsReq(_) => {
                    // DeleteTopics request verification
                }
                _ => panic!("Expected DeleteTopicsReq packet"),
            }
        }
    }
}
