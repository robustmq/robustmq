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

use super::storage_engine_engine::{
    ApiKey, ReadReq, ReadReqBody, ReadResp, ReadRespBody, ReqHeader, RespHeader, WriteReq,
    WriteReqBody, WriteResp, WriteRespBody,
};
use super::StorageError;

#[derive(Debug, PartialEq, Clone)]
pub struct StorageEngineCodec {}

#[derive(Debug, PartialEq, Clone)]
pub enum StorageEnginePacket {
    //Write
    WriteReq(WriteReq),
    WriteResp(WriteResp),

    // Read
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
    // A maximum of 1G data is transferred per request
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
            // Write
            StorageEnginePacket::WriteReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = WriteReqBody::encode_to_vec(&body);
                req_type = 1;
            }
            StorageEnginePacket::WriteResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = WriteRespBody::encode_to_vec(&body);
            }

            // Read
            StorageEnginePacket::ReadReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = ReqHeader::encode_to_vec(&header);
                body_byte = ReadReqBody::encode_to_vec(&body);
                req_type = 1;
            }
            StorageEnginePacket::ReadResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = RespHeader::encode_to_vec(&header);
                body_byte = ReadRespBody::encode_to_vec(&body);
            }
        }

        let header_len = header_byte.len();
        let body_len = body_byte.len();
        let data_len = header_len + body_len;
        if data_len > Self::MAX_SIZE {
            return Err(StorageError::PayloadSizeLimitExceeded(data_len));
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

    pub fn decode_data(
        &mut self,
        src: &mut bytes::BytesMut,
    ) -> Result<Option<StorageEnginePacket>, StorageError> {
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
            return Err(StorageError::PayloadSizeLimitExceeded(data_len));
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
            return Err(StorageError::HeaderLengthIsZero);
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

                        _ => Err(StorageError::NotAvailableRequestType(req_type)),
                    },
                    Err(e) => Err(StorageError::DecodeHeaderError(e.to_string())),
                }
            }
            // Response
            2 => match RespHeader::decode(header_body_bytes.as_ref()) {
                Ok(header) => match header.api_key() {
                    ApiKey::Write => write_resp(body_bytes, header),

                    ApiKey::Read => read_resp(body_bytes, header),

                    _ => Err(StorageError::NotAvailableRequestType(req_type)),
                },
                Err(e) => Err(StorageError::DecodeHeaderError(e.to_string())),
            },
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

fn write_req(
    body_bytes: BytesMut,
    header: ReqHeader,
) -> Result<Option<StorageEnginePacket>, StorageError> {
    match WriteReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::WriteReq(WriteReq {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(StorageError::DecodeBodyError(
            "write_req".to_string(),
            e.to_string(),
        )),
    }
}

fn write_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<StorageEnginePacket>, StorageError> {
    match WriteRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::WriteResp(WriteResp {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(StorageError::DecodeBodyError(
            "write_resp".to_string(),
            e.to_string(),
        )),
    }
}

fn read_req(
    body_bytes: BytesMut,
    header: ReqHeader,
) -> Result<Option<StorageEnginePacket>, StorageError> {
    match ReadReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::ReadReq(ReadReq {
                header: Some(header),
                body: Some(body),
            });
            Ok(Some(item))
        }
        Err(e) => Err(StorageError::DecodeBodyError(
            "read_req".to_string(),
            e.to_string(),
        )),
    }
}

fn read_resp(
    body_bytes: BytesMut,
    header: RespHeader,
) -> Result<Option<StorageEnginePacket>, StorageError> {
    match ReadRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::ReadResp(ReadResp {
                header: Some(header),
                body: Some(body),
            });
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
    use std::time::Duration;

    use futures::{SinkExt, StreamExt};
    use tokio::io;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::sleep;
    use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead, FramedWrite};

    use super::{StorageEngineCodec, StorageEnginePacket};
    use crate::storage::storage_engine_engine::{
        ApiKey, ReadReq, ReadReqBody, ReqHeader, RespHeader, WriteReq, WriteReqBody, WriteResp,
        WriteRespBody,
    };

    #[test]
    fn write_req_codec_test() {
        let header = ReqHeader {
            api_key: ApiKey::Write.into(),
        };

        let body: WriteReqBody = WriteReqBody::default();
        let req = WriteReq {
            header: Some(header),
            body: Some(body),
        };
        let source = StorageEnginePacket::WriteReq(req);

        let mut codec = StorageEngineCodec::new();
        let mut dst = bytes::BytesMut::new();
        codec.encode(source.clone(), &mut dst).unwrap();
        let target = codec.decode(&mut dst).unwrap().unwrap();
        assert_eq!(source, target);
    }

    #[test]
    fn read_codec_test() {
        let header = ReqHeader {
            api_key: ApiKey::Read.into(),
        };

        let source = StorageEnginePacket::ReadReq(ReadReq {
            header: Some(header),
            body: Some(ReadReqBody::default()),
        });

        let mut codec = StorageEngineCodec::new();
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
            let mut read_frame_stream = FramedRead::new(r_stream, StorageEngineCodec::new());
            let mut write_frame_stream = FramedWrite::new(w_stream, StorageEngineCodec::new());

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
        let mut stream: Framed<TcpStream, StorageEngineCodec> =
            Framed::new(socket, StorageEngineCodec::new());

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

    fn build_write_resp() -> StorageEnginePacket {
        let header = RespHeader {
            api_key: ApiKey::Write.into(),
            error: None,
        };

        let body = WriteRespBody::default();
        let req = WriteResp {
            header: Some(header),
            body: Some(body),
        };
        StorageEnginePacket::WriteResp(req)
    }

    fn build_write_req() -> StorageEnginePacket {
        let header = ReqHeader {
            api_key: ApiKey::Write.into(),
        };

        let body: WriteReqBody = WriteReqBody::default();
        let req = WriteReq {
            header: Some(header),
            body: Some(body),
        };
        StorageEnginePacket::WriteReq(req)
    }
}
