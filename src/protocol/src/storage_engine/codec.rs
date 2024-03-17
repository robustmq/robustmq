use super::{
    generate::protocol::{
        fetch::{FetchReq, FetchReqBody, FetchResp, FetchRespBody},
        header::{ApiKey, ApiType, Header},
        produce::{ProduceReq, ProduceReqBody, ProduceResp, ProduceRespBody},
    },
    Error,
};
use axum::error_handling;
use bytes::{buf, Buf, BufMut, BytesMut};
use common::log::{error_engine, error_meta};
use prost::Message as _;
use tokio_util::codec;

pub struct StorageEngineCodec {}

#[derive(Debug, PartialEq, Clone)]
pub enum StorageEnginePacket {
    ProduceReq(ProduceReq),
    ProduceResp(ProduceResp),
    FetchReq(FetchReq),
    FetchResp(FetchResp),
}

impl StorageEngineCodec {
    // A maximum of 1G data is transferred per request
    const MAX_SIZE: usize = 1024 * 1024 * 1024 * 8;

    pub fn new() -> StorageEngineCodec {
        return StorageEngineCodec {};
    }
}

impl codec::Encoder<StorageEnginePacket> for StorageEngineCodec {
    type Error = Error;
    fn encode(
        &mut self,
        item: StorageEnginePacket,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let mut header_byte = Vec::new();
        let mut body_byte = Vec::new();
        match item {
            StorageEnginePacket::ProduceReq(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = Header::encode_to_vec(&header);
                body_byte = ProduceReqBody::encode_to_vec(&body);
            }
            StorageEnginePacket::ProduceResp(data) => {
                let header = data.header.unwrap();
                let body = data.body.unwrap();
                header_byte = Header::encode_to_vec(&header);
                body_byte = ProduceRespBody::encode_to_vec(&body);
            }
            StorageEnginePacket::FetchReq(data) => {}
            StorageEnginePacket::FetchResp(data) => {}
        }

        let header_len = header_byte.len();
        let body_len = body_byte.len();
        let data_len = header_len + body_len;
        if data_len > Self::MAX_SIZE {
            return Err(Error::PayloadSizeLimitExceeded(data_len));
        }

        //data len + data_len + header_len + body_len
        dst.reserve(data_len + 4 + 4 + 4);

        // data len = header len + body len
        dst.put_u32(data_len as u32);

        // header len + header body
        dst.put_u32(header_len as u32);
        dst.extend_from_slice(&header_byte);

        // body len + body
        dst.put_u32(body_len as u32);
        dst.extend_from_slice(&body_byte);
        return Ok(());
    }
}

impl codec::Decoder for StorageEngineCodec {
    type Item = StorageEnginePacket;
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
        let frame_len = data_len + 4 + 4 + 4;
        if src_len < frame_len {
            src.reserve(frame_len - src_len);
            return Ok(None);
        }

        // byte data of the frame is obtained
        // frame = data len(4) + header len(4) + header body(N) + body len(4) + body(N)
        // If the amount of data is sufficient, the data is taken from the buf and converted into frames.
        // Also truncate buf (split_to will truncate)
        let frame_bytes = src.split_to(frame_len);

        // length of the header is parsed
        position = position + 4;
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
        position = position + 4;
        let mut header_body_bytes = BytesMut::with_capacity(header_len);
        header_body_bytes.extend_from_slice(&frame_bytes[position..(position + header_len)]);

        // Parse to get the length of the body
        position = position + header_len;
        let mut body_len_bytes = BytesMut::with_capacity(4);
        body_len_bytes.extend_from_slice(&frame_bytes[position..(position + 4)]);
        let body_len = u32::from_be_bytes([
            body_len_bytes[0],
            body_len_bytes[1],
            body_len_bytes[2],
            body_len_bytes[3],
        ]) as usize;

        // Parse the contents of the body
        position = position + 4;
        let mut body_bytes = BytesMut::with_capacity(body_len);
        body_bytes.extend_from_slice(&frame_bytes[position..(position + body_len)]);

        // Build structured data from the contents of the body and header
        match Header::decode(header_body_bytes.as_ref()) {
            Ok(header) => match header.api_key() {
                ApiKey::Produce => match header.api_type() {
                    ApiType::Request => return produce_req(body_bytes, header),
                    ApiType::Response => return produce_resp(body_bytes, header),
                },
                ApiKey::Consume => match header.api_type() {
                    ApiType::Request => return fetch_req(body_bytes, header),
                    ApiType::Response => return fetch_resp(body_bytes, header),
                },
            },
            Err(e) => {
                return Err(Error::DecodeHeaderError(e.to_string()));
            }
        }
        Ok(None)
    }
}

fn produce_req(body_bytes: BytesMut, header: Header) -> Result<Option<StorageEnginePacket>, Error> {
    match ProduceReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::ProduceReq(ProduceReq {
                header: Some(header),
                body: Some(body),
            });
            return Ok(Some(item));
        }
        Err(e) => {
            return Err(Error::DecodeBodyError(
                "produce_req".to_string(),
                e.to_string(),
            ));
        }
    }
}

fn produce_resp(
    body_bytes: BytesMut,
    header: Header,
) -> Result<Option<StorageEnginePacket>, Error> {
    match ProduceRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::ProduceResp(ProduceResp {
                header: Some(header),
                body: Some(body),
            });
            return Ok(Some(item));
        }
        Err(e) => {
            return Err(Error::DecodeBodyError(
                "produce_resp".to_string(),
                e.to_string(),
            ));
        }
    }
}

fn fetch_req(body_bytes: BytesMut, header: Header) -> Result<Option<StorageEnginePacket>, Error> {
    match FetchReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::FetchReq(FetchReq {
                header: Some(header),
                body: Some(body),
            });
            return Ok(Some(item));
        }
        Err(e) => {
            return Err(Error::DecodeBodyError(
                "fetch_req".to_string(),
                e.to_string(),
            ));
        }
    }
}

fn fetch_resp(body_bytes: BytesMut, header: Header) -> Result<Option<StorageEnginePacket>, Error> {
    match FetchRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::FetchResp(FetchResp {
                header: Some(header),
                body: Some(body),
            });
            return Ok(Some(item));
        }
        Err(e) => {
            return Err(Error::DecodeBodyError(
                "fetch_resp".to_string(),
                e.to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio_util::codec::{Decoder, Encoder, Framed};

    use crate::storage_engine::generate::protocol::{
        header::{ApiKey, ApiType, ApiVersion, Header, RequestCommon, ResponseCommon},
        produce::{ProduceReq, ProduceReqBody, ProduceResp, ProduceRespBody},
    };

    use super::{StorageEngineCodec, StorageEnginePacket};

    #[test]
    fn codec_test() {
        let mut codec = StorageEngineCodec::new();
        let source = build_produce_req();
        let mut dst = bytes::BytesMut::new();
        codec.encode(source.clone(), &mut dst).unwrap();
        let target = codec.decode(&mut dst).unwrap().unwrap();
        assert_eq!(source, target);
    }

    #[tokio::test]
    async fn storage_engine_frame_server() {
        let ip = "127.0.0.1:1228";
        let listener = TcpListener::bind(ip).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let mut stream = Framed::new(stream, StorageEngineCodec::new());
            tokio::spawn(async move {
                while let Some(Ok(data)) = stream.next().await {
                    println!("Server Receive: {:?}", data);

                    // 发送的消息也只需要发送消息主体，不需要提供长度
                    // Framed/LengthDelimitedCodec 会自动计算并添加
                    //    let response = &data[0..5];
                    let resp = build_produce_resp();
                    stream.send(resp).await.unwrap();
                }
            });
        }
    }

    #[tokio::test]
    async fn storage_engine_frame_client() {
        let socket = TcpStream::connect("127.0.0.1:2228").await.unwrap();
        let mut stream: Framed<TcpStream, StorageEngineCodec> =
            Framed::new(socket, StorageEngineCodec::new());

        let _ = stream.send(build_produce_req()).await;

        if let Some(res) = stream.next().await {
            match res {
                Ok(da) => {
                    println!("{:?}", da);
                }
                Err(e) => {
                    println!("{}", e.to_string());
                }
            }
        }
    }

    fn build_produce_resp() -> StorageEnginePacket {
        let header = Header {
            api_key: ApiKey::Produce.into(),
            api_type: ApiType::Response.into(),
            api_version: ApiVersion::V0.into(),
            request: None,
            response: Some(ResponseCommon { correlation_id: 33 }),
        };

        let body = ProduceRespBody {};
        let req = ProduceResp {
            header: Some(header),
            body: Some(body),
        };
        return StorageEnginePacket::ProduceResp(req);
    }

    fn build_produce_req() -> StorageEnginePacket {
        let header = Header {
            api_key: ApiKey::Produce.into(),
            api_type: ApiType::Request.into(),
            api_version: ApiVersion::V0.into(),
            request: Some(RequestCommon {
                correlation_id: 3,
                client_id: "testsssss".to_string(),
            }),
            response: None,
        };

        let body = ProduceReqBody {
            transactional_id: 1,
            acks: 1,
            timeout_ms: 60000,
            topic_data: None,
        };
        let req = ProduceReq {
            header: Some(header),
            body: Some(body),
        };
        return StorageEnginePacket::ProduceReq(req);
    }
}
