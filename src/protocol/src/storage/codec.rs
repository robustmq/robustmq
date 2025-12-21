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

use super::protocol::{ApiKey, ReadReq, ReadResp, WriteReq, WriteResp};
use super::StorageError;
use bytes::{BufMut, BytesMut};
use std::fmt;
use tokio_util::codec;

#[derive(Debug, PartialEq, Clone)]
pub struct StorageEngineCodec {}

#[derive(Clone, Debug, PartialEq)]
pub enum StorageEnginePacket {
    WriteReq(WriteReq),
    WriteResp(WriteResp),
    ReadReq(ReadReq),
    ReadResp(ReadResp),
}

impl fmt::Display for StorageEnginePacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StorageEnginePacket::WriteReq(_) => write!(f, "WriteReq"),
            StorageEnginePacket::WriteResp(_) => write!(f, "WriteResp"),
            StorageEnginePacket::ReadReq(_) => write!(f, "ReadReq"),
            StorageEnginePacket::ReadResp(_) => write!(f, "ReadResp"),
        }
    }
}

impl Default for StorageEngineCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngineCodec {
    const MAX_SIZE: usize = 1024 * 1024 * 1024 * 8;

    pub fn new() -> StorageEngineCodec {
        StorageEngineCodec {}
    }

    pub fn encode_data(
        &self,
        item: StorageEnginePacket,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), StorageError> {
        let header_byte;
        let body_byte;
        let mut req_type = 2;

        match item {
            StorageEnginePacket::WriteReq(data) => {
                header_byte = data.header.encode();
                body_byte = data.body.encode();
                req_type = 1;
            }
            StorageEnginePacket::WriteResp(data) => {
                header_byte = data.header.encode();
                body_byte = data.body.encode();
            }
            StorageEnginePacket::ReadReq(data) => {
                header_byte = data.header.encode();
                body_byte = data.body.encode();
                req_type = 1;
            }
            StorageEnginePacket::ReadResp(data) => {
                header_byte = data.header.encode();
                body_byte = data.body.encode();
            }
        }

        let header_len = header_byte.len();
        let body_len = body_byte.len();
        let data_len = header_len + body_len;
        if data_len > Self::MAX_SIZE {
            return Err(StorageError::PayloadSizeLimitExceeded(data_len));
        }

        dst.reserve(data_len + 1 + 4 + 4 + 4);

        dst.put_u32(data_len as u32);
        dst.put_u8(req_type);
        dst.put_u32(header_len as u32);
        dst.extend_from_slice(&header_byte);
        dst.put_u32(body_len as u32);
        dst.extend_from_slice(&body_byte);
        Ok(())
    }

    pub fn decode_data(
        &mut self,
        src: &mut bytes::BytesMut,
    ) -> Result<Option<StorageEnginePacket>, StorageError> {
        let src_len = src.len();
        if src_len < 4 {
            return Ok(None);
        }

        let mut position = 0;
        let mut data_len_bytes = BytesMut::with_capacity(4);
        data_len_bytes.extend_from_slice(&src[..4]);
        let data_len = u32::from_be_bytes([
            data_len_bytes[0],
            data_len_bytes[1],
            data_len_bytes[2],
            data_len_bytes[3],
        ]) as usize;
        if data_len > Self::MAX_SIZE {
            return Err(StorageError::PayloadSizeLimitExceeded(data_len));
        }

        let frame_len = data_len + 1 + 4 + 4 + 4;
        if src_len < frame_len {
            src.reserve(frame_len - src_len);
            return Ok(None);
        }

        let frame_bytes = src.split_to(frame_len);

        position += 4;
        let mut req_type_bytes = BytesMut::with_capacity(4);
        req_type_bytes.extend_from_slice(&frame_bytes[position..(position + 1)]);
        let req_type: u8 = u8::from_be_bytes([req_type_bytes[0]]);

        position += 1;
        let mut header_len_bytes = BytesMut::with_capacity(4);
        header_len_bytes.extend_from_slice(&frame_bytes[position..(position + 4)]);
        let header_len = u32::from_be_bytes([
            header_len_bytes[0],
            header_len_bytes[1],
            header_len_bytes[2],
            header_len_bytes[3],
        ]) as usize;
        if header_len == 0 {
            return Err(StorageError::HeaderLengthIsZero);
        }

        position += 4;
        let mut header_body_bytes = BytesMut::with_capacity(header_len);
        header_body_bytes.extend_from_slice(&frame_bytes[position..(position + header_len)]);

        position += header_len;
        let mut body_len_bytes = BytesMut::with_capacity(4);
        body_len_bytes.extend_from_slice(&frame_bytes[position..(position + 4)]);
        let body_len = u32::from_be_bytes([
            body_len_bytes[0],
            body_len_bytes[1],
            body_len_bytes[2],
            body_len_bytes[3],
        ]) as usize;

        position += 4;
        let mut body_bytes = BytesMut::with_capacity(body_len);
        body_bytes.extend_from_slice(&frame_bytes[position..(position + body_len)]);

        match req_type {
            1 => {
                use super::protocol::ReqHeader;
                match ReqHeader::decode(&header_body_bytes) {
                    Ok(header) => match header.api_key {
                        ApiKey::Write => decode_write_req(&body_bytes, header),
                        ApiKey::Read => decode_read_req(&body_bytes, header),
                        _ => Err(StorageError::NotAvailableRequestType(req_type)),
                    },
                    Err(e) => Err(StorageError::DecodeHeaderError(e.to_string())),
                }
            }
            2 => {
                use super::protocol::RespHeader;
                match RespHeader::decode(&header_body_bytes) {
                    Ok(header) => match header.api_key {
                        ApiKey::Write => decode_write_resp(&body_bytes, header),
                        ApiKey::Read => decode_read_resp(&body_bytes, header),
                        _ => Err(StorageError::NotAvailableRequestType(req_type)),
                    },
                    Err(e) => Err(StorageError::DecodeHeaderError(e.to_string())),
                }
            }
            _ => Err(StorageError::NotAvailableRequestType(req_type)),
        }
    }
}

impl codec::Encoder<StorageEnginePacket> for StorageEngineCodec {
    type Error = StorageError;
    fn encode(
        &mut self,
        item: StorageEnginePacket,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        self.encode_data(item, dst)
    }
}

impl codec::Decoder for StorageEngineCodec {
    type Item = StorageEnginePacket;
    type Error = StorageError;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode_data(src)
    }
}

fn decode_write_req(
    body_bytes: &[u8],
    header: super::protocol::ReqHeader,
) -> Result<Option<StorageEnginePacket>, StorageError> {
    use super::protocol::WriteReqBody;
    match WriteReqBody::decode(body_bytes) {
        Ok(body) => {
            let item = StorageEnginePacket::WriteReq(WriteReq { header, body });
            Ok(Some(item))
        }
        Err(e) => Err(StorageError::DecodeBodyError(
            "write_req".to_string(),
            e.to_string(),
        )),
    }
}

fn decode_write_resp(
    body_bytes: &[u8],
    header: super::protocol::RespHeader,
) -> Result<Option<StorageEnginePacket>, StorageError> {
    use super::protocol::WriteRespBody;
    match WriteRespBody::decode(body_bytes) {
        Ok(body) => {
            let item = StorageEnginePacket::WriteResp(WriteResp { header, body });
            Ok(Some(item))
        }
        Err(e) => Err(StorageError::DecodeBodyError(
            "write_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn decode_read_req(
    body_bytes: &[u8],
    header: super::protocol::ReqHeader,
) -> Result<Option<StorageEnginePacket>, StorageError> {
    use super::protocol::ReadReqBody;
    match ReadReqBody::decode(body_bytes) {
        Ok(body) => {
            let item = StorageEnginePacket::ReadReq(ReadReq { header, body });
            Ok(Some(item))
        }
        Err(e) => Err(StorageError::DecodeBodyError(
            "read_req".to_string(),
            e.to_string(),
        )),
    }
}

fn decode_read_resp(
    body_bytes: &[u8],
    header: super::protocol::RespHeader,
) -> Result<Option<StorageEnginePacket>, StorageError> {
    use super::protocol::ReadRespBody;
    match ReadRespBody::decode(body_bytes) {
        Ok(body) => {
            let item = StorageEnginePacket::ReadResp(ReadResp { header, body });
            Ok(Some(item))
        }
        Err(e) => Err(StorageError::DecodeBodyError(
            "read_resp".to_string(),
            e.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{StorageEngineCodec, StorageEnginePacket};
    use crate::storage::protocol::*;

    #[test]
    fn write_req_codec_test() {
        let header = ReqHeader::new(ApiKey::Write);
        let body = WriteReqBody::default();
        let req = WriteReq { header, body };
        let source = StorageEnginePacket::WriteReq(req);

        let mut codec = StorageEngineCodec::new();
        let mut dst = bytes::BytesMut::new();
        codec.encode_data(source.clone(), &mut dst).unwrap();
        let target = codec.decode_data(&mut dst).unwrap().unwrap();

        assert_eq!(source, target);
    }

    #[test]
    fn read_req_codec_test() {
        let header = ReqHeader::new(ApiKey::Read);
        let body = ReadReqBody::default();
        let source = StorageEnginePacket::ReadReq(ReadReq { header, body });

        let mut codec = StorageEngineCodec::new();
        let mut dst = bytes::BytesMut::new();
        codec.encode_data(source.clone(), &mut dst).unwrap();
        let target = codec.decode_data(&mut dst).unwrap().unwrap();

        assert_eq!(source, target);
    }

    #[test]
    fn write_resp_codec_test() {
        let header = RespHeader::new(ApiKey::Write);
        let body = WriteRespBody::default();
        let resp = WriteResp { header, body };
        let source = StorageEnginePacket::WriteResp(resp);

        let mut codec = StorageEngineCodec::new();
        let mut dst = bytes::BytesMut::new();
        codec.encode_data(source.clone(), &mut dst).unwrap();
        let target = codec.decode_data(&mut dst).unwrap().unwrap();

        assert_eq!(source, target);
    }

    #[test]
    fn read_resp_codec_test() {
        let header = RespHeader::new(ApiKey::Read);
        let body = ReadRespBody::default();
        let resp = ReadResp { header, body };
        let source = StorageEnginePacket::ReadResp(resp);

        let mut codec = StorageEngineCodec::new();
        let mut dst = bytes::BytesMut::new();
        codec.encode_data(source.clone(), &mut dst).unwrap();
        let target = codec.decode_data(&mut dst).unwrap().unwrap();

        assert_eq!(source, target);
    }
}
