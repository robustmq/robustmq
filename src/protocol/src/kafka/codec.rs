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

use crate::kafka::packet::{KafkaPacket, KafkaPacketWrapper};
use bytes::{Buf, BufMut, BytesMut};
use common_base::error::common::CommonError;
use kafka_protocol::{
    messages::{
        ApiVersionsRequest, FetchRequest, ListOffsetsRequest, MetadataRequest, ProduceRequest,
        RequestHeader,
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
            Error::new(
                ErrorKind::InvalidData,
                format!("Header decode failed: {}", e),
            )
        })?;

        println!("{:?}", header);

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

            18 => {
                let req = ApiVersionsRequest::decode(&mut buf, header.request_api_version)?;
                KafkaPacket::ApiVersionReq(req)
            }

            _ => {
                return Err(CommonError::NotSupportKafkaRequest(header.request_api_key));
            }
        };
        Ok(Some(KafkaPacketWrapper {
            header,
            packet: req,
        }))
    }

    pub fn encode_data(
        &mut self,
        wrapper: KafkaPacketWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), CommonError> {
        let mut header_bytes = BytesMut::new();
        wrapper.header.encode(&mut header_bytes, 2)?;
        let mut body_bytes = BytesMut::new();
        match wrapper.packet {
            KafkaPacket::ProduceResponse(rep) => {
                rep.encode(&mut body_bytes, wrapper.header.request_api_version)?;
            }
            KafkaPacket::FetchResponse(rep) => {
                rep.encode(&mut body_bytes, wrapper.header.request_api_version)?;
            }
            KafkaPacket::ListOffsetsResponse(rep) => {
                rep.encode(&mut body_bytes, wrapper.header.request_api_version)?;
            }
            KafkaPacket::MetadataResponse(rep) => {
                rep.encode(&mut body_bytes, wrapper.header.request_api_version)?;
            }
            KafkaPacket::ApiVersionResponse(rep) => {
                rep.encode(&mut body_bytes, wrapper.header.request_api_version)?;
            }
            _ => {
                return Err(CommonError::NotSupportKafkaEncodePacket(format!(
                    "{:?}",
                    wrapper.packet
                )));
            }
        }
        let total_len = header_bytes.len() + body_bytes.len();
        let len_byte = total_len.to_be_bytes();
        buffer.put_slice(&len_byte);
        buffer.put_slice(&header_bytes);
        buffer.put_slice(&header_bytes);

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

    #[tokio::test]
    async fn protocol_decode() {}
}
