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

use std::fmt;

use bytes::{BufMut, BytesMut};
use prost::Message as _;
use tokio_util::codec;

use super::journal_engine::{
    ApiKey, CreateShardReq, CreateShardReqBody, CreateShardResp, CreateShardRespBody,
    DeleteShardReq, DeleteShardReqBody, DeleteShardResp, DeleteShardRespBody, FetchOffsetReq,
    FetchOffsetReqBody, FetchOffsetResp, FetchOffsetRespBody, GetClusterMetadataReq,
    GetClusterMetadataResp, GetClusterMetadataRespBody, GetShardMetadataReq,
    GetShardMetadataReqBody, GetShardMetadataResp, GetShardMetadataRespBody, ListShardReq,
    ListShardReqBody, ListShardResp, ListShardRespBody, ReadReq, ReadReqBody, ReadResp,
    ReadRespBody, ReqHeader, RespHeader, WriteReq, WriteReqBody, WriteResp, WriteRespBody,
};
use super::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct JournalServerCodec {}

#[derive(Debug, PartialEq, Clone)]
pub enum JournalEnginePacket {
    //Write
    WriteReq(WriteReq),
    WriteResp(WriteResp),

    // Read
    ReadReq(ReadReq),
    ReadResp(ReadResp),

    // GetClusterMetadata
    GetClusterMetadataReq(GetClusterMetadataReq),
    GetClusterMetadataResp(GetClusterMetadataResp),

    // GetShardMetadata
    GetShardMetadataReq(GetShardMetadataReq),
    GetShardMetadataResp(GetShardMetadataResp),

    // FetchOffset
    FetchOffsetReq(FetchOffsetReq),
    FetchOffsetResp(FetchOffsetResp),

    // CreateShard
    CreateShardReq(CreateShardReq),
    CreateShardResp(CreateShardResp),

    // DeleteShard
    DeleteShardReq(DeleteShardReq),
    DeleteShardResp(DeleteShardResp),

    // ListShard
    ListShardReq(ListShardReq),
    ListShardResp(ListShardResp),
}

impl fmt::Display for JournalEnginePacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JournalEnginePacket::WriteReq(_) => write!(f, "WriteReq"),
            JournalEnginePacket::WriteResp(_) => write!(f, "WriteResp"),
            JournalEnginePacket::ReadReq(_) => write!(f, "ReadReq"),
            JournalEnginePacket::ReadResp(_) => write!(f, "ReadResp"),
            JournalEnginePacket::GetClusterMetadataReq(_) => write!(f, "GetClusterMetadataReq"),
            JournalEnginePacket::GetClusterMetadataResp(_) => write!(f, "GetClusterMetadataResp"),
            JournalEnginePacket::GetShardMetadataReq(_) => write!(f, "GetShardMetadataReq"),
            JournalEnginePacket::GetShardMetadataResp(_) => write!(f, "GetShardMetadataResp"),
            JournalEnginePacket::FetchOffsetReq(_) => write!(f, "FetchOffsetReq"),
            JournalEnginePacket::FetchOffsetResp(_) => write!(f, "FetchOffsetResp"),
            JournalEnginePacket::CreateShardReq(_) => write!(f, "CreateShardReq"),
            JournalEnginePacket::CreateShardResp(_) => write!(f, "CreateShardResp"),
            JournalEnginePacket::DeleteShardReq(_) => write!(f, "DeleteShardReq"),
            JournalEnginePacket::DeleteShardResp(_) => write!(f, "DeleteShardResp"),
            JournalEnginePacket::ListShardReq(_) => write!(f, "ListShardReq"),
            JournalEnginePacket::ListShardResp(_) => write!(f, "ListShardResp"),
        }
    }
}

impl Default for JournalServerCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl JournalServerCodec {
    // A maximum of 1G data is transferred per request
    const MAX_SIZE: usize = 1024 * 1024 * 1024 * 8;

    pub fn new() -> JournalServerCodec {
        JournalServerCodec {}
    }
}

impl codec::Encoder<JournalEnginePacket> for JournalServerCodec {
    type Error = Error;
    fn encode(
        &mut self,
        item: JournalEnginePacket,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let header_byte;
        let body_byte;
        let mut req_type = 2;
        match item {
            // Write
            JournalEnginePacket::WriteReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = WriteReqBody::encode_to_vec(&body);
                req_type = 1;
            }
            JournalEnginePacket::WriteResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = WriteRespBody::encode_to_vec(&body);
            }

            // Read
            JournalEnginePacket::ReadReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = ReadReqBody::encode_to_vec(&body);
                req_type = 1;
            }
            JournalEnginePacket::ReadResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = ReadRespBody::encode_to_vec(&body);
            }

            // GetClusterMetadata
            JournalEnginePacket::GetClusterMetadataReq(data) => {
                let header = data.header.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = Vec::new();
                req_type = 1;
            }
            JournalEnginePacket::GetClusterMetadataResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = GetClusterMetadataRespBody::encode_to_vec(&body);
            }

            // GetShardMetadata
            JournalEnginePacket::GetShardMetadataReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = GetShardMetadataReqBody::encode_to_vec(&body);
                req_type = 1;
            }
            JournalEnginePacket::GetShardMetadataResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = GetShardMetadataRespBody::encode_to_vec(&body);
            }

            // FetchOffset
            JournalEnginePacket::FetchOffsetReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = FetchOffsetReqBody::encode_to_vec(&body);
                req_type = 1;
            }
            JournalEnginePacket::FetchOffsetResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = FetchOffsetRespBody::encode_to_vec(&body);
            }

            // CreateShard
            JournalEnginePacket::CreateShardReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = CreateShardReqBody::encode_to_vec(&body);
                req_type = 1;
            }
            JournalEnginePacket::CreateShardResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = CreateShardRespBody::encode_to_vec(&body);
            }

            // DeleteShard
            JournalEnginePacket::DeleteShardReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = DeleteShardReqBody::encode_to_vec(&body);
                req_type = 1;
            }
            JournalEnginePacket::DeleteShardResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = DeleteShardRespBody::encode_to_vec(&body);
            }

            // ListShard
            JournalEnginePacket::ListShardReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = ListShardReqBody::encode_to_vec(&body);
                req_type = 1;
            }

            JournalEnginePacket::ListShardResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = ListShardRespBody::encode_to_vec(&body);
            }
        }

        let header_len = header_byte.len();
        let body_len = body_byte.len();
        let data_len = header_len + body_len;
        if data_len > Self::MAX_SIZE {
            return Err(Error::PayloadSizeLimitExceeded(data_len));
        }

        //data len + data_len  + req_type + header_len + body_len
        dst.reserve(data_len + 1 + 4 + 4 + 4);

        // data len = header len + body len
        dst.put_u32(data_len as u32);

        // req type
        dst.put_u8(req_type);

        // header len + header body
        dst.put_u32(header_len as u32);
        dst.extend_from_slice(&header_byte);

        // body len + body
        dst.put_u32(body_len as u32);
        dst.extend_from_slice(&body_byte);
        Ok(())
    }
}

impl codec::Decoder for JournalServerCodec {
    type Item = JournalEnginePacket;
    type Error = Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let src_len = src.len();
        if src_len < 4 {
            return Ok(None);
        }

        // Starting at position=0, go back 4 bits to get the total length of the package
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
            return Err(Error::PayloadSizeLimitExceeded(data_len));
        }

        // Total frame length = total packet length (data_len) + Total length (4) + header length (4) + body length (4)
        let frame_len = data_len + 1 + 4 + 4 + 4;
        if src_len < frame_len {
            src.reserve(frame_len - src_len);
            return Ok(None);
        }

        // byte data of the frame is obtained
        // frame = data len(4) + header len(4) + header body(N) + body len(4) + body(N)
        // If the amount of data is sufficient, the data is taken from the buf and converted into frames.
        // Also truncate buf (split_to will truncate)
        let frame_bytes = src.split_to(frame_len);

        // parsed req type
        position += 4;
        let mut req_type_bytes = BytesMut::with_capacity(4);
        req_type_bytes.extend_from_slice(&frame_bytes[position..(position + 1)]);
        let req_type: u8 = u8::from_be_bytes([req_type_bytes[0]]);

        // length of the header is parsed
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
            return Err(Error::HeaderLengthIsZero);
        }

        // Parse the contents of the header
        position += 4;
        let mut header_body_bytes = BytesMut::with_capacity(header_len);
        header_body_bytes.extend_from_slice(&frame_bytes[position..(position + header_len)]);

        // Parse to get the length of the body
        position += header_len;
        let mut body_len_bytes = BytesMut::with_capacity(4);
        body_len_bytes.extend_from_slice(&frame_bytes[position..(position + 4)]);
        let body_len = u32::from_be_bytes([
            body_len_bytes[0],
            body_len_bytes[1],
            body_len_bytes[2],
            body_len_bytes[3],
        ]) as usize;

        // Parse the contents of the body
        position += 4;
        let mut body_bytes = BytesMut::with_capacity(body_len);
        body_bytes.extend_from_slice(&frame_bytes[position..(position + body_len)]);

        match req_type {
            // Request
            1 => {
                // Build structured data from the contents of the body and header
                match ReqHeader::decode(header_body_bytes.as_ref()) {
                    Ok(header) => match header.api_key() {
                        ApiKey::Write => write_req(body_bytes, header),

                        ApiKey::Read => read_req(body_bytes, header),

                        ApiKey::GetClusterMetadata => get_cluster_metadata_req(body_bytes, header),

                        ApiKey::GetShardMetadata => get_shard_metadata_req(body_bytes, header),

                        ApiKey::FetchOffset => fetch_offset_req(body_bytes, header),

                        ApiKey::CreateShard => create_shard_req(body_bytes, header),

                        ApiKey::DeleteShard => delete_shard_req(body_bytes, header),

                        _ => Err(Error::NotAvailableRequestType(req_type)),
                    },
                    Err(e) => Err(Error::DecodeHeaderError(e.to_string())),
                }
            }
            // Response
            2 => match RespHeader::decode(header_body_bytes.as_ref()) {
                Ok(header) => match header.api_key() {
                    ApiKey::Write => write_resp(body_bytes, header),

                    ApiKey::Read => read_resp(body_bytes, header),

                    ApiKey::GetClusterMetadata => get_cluster_metadata_resp(body_bytes, header),

                    ApiKey::GetShardMetadata => get_shard_metadata_resp(body_bytes, header),

                    ApiKey::FetchOffset => fetch_offset_resp(body_bytes, header),

                    ApiKey::CreateShard => create_shard_resp(body_bytes, header),

                    ApiKey::DeleteShard => delete_shard_resp(body_bytes, header),

                    _ => Err(Error::NotAvailableRequestType(req_type)),
                },
                Err(e) => Err(Error::DecodeHeaderError(e.to_string())),
            },
            _ => Err(Error::NotAvailableRequestType(req_type)),
        }
    }
}

fn create_shard_req(
    body_bytes: BytesMut,
    header: ReqHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match CreateShardReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::CreateShardReq(CreateShardReq {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "write_req".to_string(),
            e.to_string(),
        )),
    }
}

fn create_shard_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match CreateShardRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::CreateShardResp(CreateShardResp {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "write_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn delete_shard_req(
    body_bytes: BytesMut,
    header: ReqHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match DeleteShardReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::DeleteShardReq(DeleteShardReq {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "write_req".to_string(),
            e.to_string(),
        )),
    }
}

fn delete_shard_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match DeleteShardRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::DeleteShardResp(DeleteShardResp {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "write_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn write_req(
    body_bytes: BytesMut,
    header: ReqHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match WriteReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::WriteReq(WriteReq {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "write_req".to_string(),
            e.to_string(),
        )),
    }
}

fn write_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match WriteRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::WriteResp(WriteResp {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "write_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn read_req(body_bytes: BytesMut, header: ReqHeader) -> Result<Option<JournalEnginePacket>, Error> {
    match ReadReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::ReadReq(ReadReq {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "read_req".to_string(),
            e.to_string(),
        )),
    }
}

fn read_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match ReadRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::ReadResp(ReadResp {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "read_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn get_cluster_metadata_req(
    _: BytesMut,
    header: ReqHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    Ok(Some(JournalEnginePacket::GetClusterMetadataReq(
        GetClusterMetadataReq {
            header: Some(header),
        },
    )))
}
fn get_cluster_metadata_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match GetClusterMetadataRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::GetClusterMetadataResp(GetClusterMetadataResp {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "get_cluster_metadata_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn get_shard_metadata_req(
    body_bytes: BytesMut,
    header: ReqHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match GetShardMetadataReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::GetShardMetadataReq(GetShardMetadataReq {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "get_shard_metadata_req".to_string(),
            e.to_string(),
        )),
    }
}

fn get_shard_metadata_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match GetShardMetadataRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::GetShardMetadataResp(GetShardMetadataResp {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "get_shard_metadata_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn fetch_offset_req(
    body_bytes: BytesMut,
    header: ReqHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match FetchOffsetReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::FetchOffsetReq(FetchOffsetReq {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "fetch_offset_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn fetch_offset_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<JournalEnginePacket>, Error> {
    match FetchOffsetRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = JournalEnginePacket::FetchOffsetResp(FetchOffsetResp {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(Error::DecodeBodyError(
            "fetch_offset_resp".to_string(),
            e.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{SinkExt, StreamExt};
    use tokio::io;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::sleep;
    use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead, FramedWrite};

    use super::{JournalEnginePacket, JournalServerCodec};
    use crate::journal_server::journal_engine::{
        ApiKey, ApiVersion, GetClusterMetadataReq, ReadReq, ReadReqBody, ReqHeader, RespHeader,
        WriteReq, WriteReqBody, WriteResp, WriteRespBody,
    };

    #[test]
    fn write_req_codec_test() {
        let header = ReqHeader {
            api_key: ApiKey::Write.into(),
            api_version: ApiVersion::V0.into(),
        };

        let body: WriteReqBody = WriteReqBody::default();
        let req = WriteReq {
            header: Some(header),
            body: Some(body),
        };
        let source = JournalEnginePacket::WriteReq(req);

        let mut codec = JournalServerCodec::new();
        let mut dst = bytes::BytesMut::new();
        codec.encode(source.clone(), &mut dst).unwrap();
        let target = codec.decode(&mut dst).unwrap().unwrap();
        assert_eq!(source, target);
    }

    #[test]
    fn get_cluster_metadata_codec_test() {
        let header = ReqHeader {
            api_key: ApiKey::GetClusterMetadata.into(),
            api_version: ApiVersion::V0.into(),
        };

        let req = GetClusterMetadataReq {
            header: Some(header),
        };
        let source = JournalEnginePacket::GetClusterMetadataReq(req);

        let mut codec = JournalServerCodec::new();
        let mut dst = bytes::BytesMut::new();
        codec.encode(source.clone(), &mut dst).unwrap();
        let target = codec.decode(&mut dst).unwrap().unwrap();
        assert_eq!(source, target);
    }

    #[test]
    fn read_codec_test() {
        let header = ReqHeader {
            api_key: ApiKey::Read.into(),
            api_version: ApiVersion::V0.into(),
        };

        let source = JournalEnginePacket::ReadReq(ReadReq {
            header: Some(header),
            body: Some(ReadReqBody::default()),
        });

        let mut codec = JournalServerCodec::new();
        let mut dst = bytes::BytesMut::new();
        codec.encode(source.clone(), &mut dst).unwrap();
        let target = codec.decode(&mut dst).unwrap().unwrap();
        assert_eq!(source, target);
    }

    #[tokio::test]
    async fn storage_engine_frame_server() {
        let req_pkg = build_write_req();
        let resp_pkg = build_write_resp();

        let resp = resp_pkg.clone();
        tokio::spawn(async move {
            let ip = "127.0.0.1:7228";
            let listener = TcpListener::bind(ip).await.unwrap();

            let (stream, _) = listener.accept().await.unwrap();
            let (r_stream, w_stream) = io::split(stream);
            let mut read_frame_stream = FramedRead::new(r_stream, JournalServerCodec::new());
            let mut write_frame_stream = FramedWrite::new(w_stream, JournalServerCodec::new());

            while let Some(Ok(data)) = read_frame_stream.next().await {
                println!("Server Receive: {:?}", data);

                // 发送的消息也只需要发送消息主体，不需要提供长度
                // Framed/LengthDelimitedCodec 会自动计算并添加
                //    let response = &data[0..5];
                write_frame_stream.send(resp.clone()).await.unwrap();
                write_frame_stream.send(resp.clone()).await.unwrap();
                write_frame_stream.send(resp.clone()).await.unwrap();
                write_frame_stream.send(resp.clone()).await.unwrap();
            }
        });

        sleep(Duration::from_secs(5)).await;

        let socket = TcpStream::connect("127.0.0.1:7228").await.unwrap();
        let mut stream: Framed<TcpStream, JournalServerCodec> =
            Framed::new(socket, JournalServerCodec::new());

        let _ = stream.send(req_pkg).await;

        if let Some(res) = stream.next().await {
            match res {
                Ok(da) => {
                    assert_eq!(da, resp_pkg);
                }
                Err(e) => {
                    println!("{}", e);
                    assert!(false);
                }
            }
        }
    }

    fn build_write_resp() -> JournalEnginePacket {
        let header = RespHeader {
            api_key: ApiKey::Write.into(),
            api_version: ApiVersion::V0.into(),
            error: None,
        };

        let body = WriteRespBody::default();
        let req = WriteResp {
            header: Some(header),
            body: Some(body),
        };
        JournalEnginePacket::WriteResp(req)
    }

    fn build_write_req() -> JournalEnginePacket {
        let header = ReqHeader {
            api_key: ApiKey::Write.into(),
            api_version: ApiVersion::V0.into(),
        };

        let body: WriteReqBody = WriteReqBody::default();
        let req = WriteReq {
            header: Some(header),
            body: Some(body),
        };
        JournalEnginePacket::WriteReq(req)
    }
}
