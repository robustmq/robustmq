use super::storage::{
    ApiKey, ApiType, FetchReq, FetchReqBody, FetchResp, FetchRespBody, Header, ProduceReq,
    ProduceReqBody, ProduceResp, ProduceRespBody,
};
use crate::mqtt::Error;
use bytes::{buf, BufMut};
use prost::Message as _;
use tokio_util::codec;

pub struct StorageEngineCodec {}

#[derive(Debug)]
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
            return Err(Error::InvalidProtocol);
        }

        println!("encode data_len={}", data_len);
        println!("encode header_len={}", header_len);
        println!("encode body_len={}", body_len);
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

        // 从 position=0 开始，向后取4位，得到包的总长度
        let mut position = 0;
        let mut data_len_bytes = [0u8; 4];
        data_len_bytes.copy_from_slice(&src[..4]);
        let data_len = u32::from_be_bytes(data_len_bytes) as usize;
        println!("decode data_len={}", data_len);
        if data_len > Self::MAX_SIZE {
            return Err(Error::InvalidProtocol);
        }

        // 帧的总长度 = 包的总长度(data_len) + 总长度(4) + 头长度(4) + body长度(4)
        let frame_len = data_len + 4 + 4 + 4;
        println!("decode frame_len={}", frame_len);
        if src_len < frame_len {
            src.reserve(frame_len - src_len);
            return Ok(None);
        }

        // 得到帧的byte数据
        // frame = data len(4) + header len(4) + header body(N) + body len(4) + body(N)
        let frame_bytes = src.split_to(frame_len);

        // 解析得到 header 的长度
        position = position + 4;
        let mut header_len_bytes = [0u8; 4];
        header_len_bytes.copy_from_slice(&src[position..(position + 4)]);
        let header_len = u32::from_be_bytes(header_len_bytes) as usize;
        println!("decode header_len={}", header_len);
        if header_len == 0 {
            return Err(Error::InvalidProtocol);
        }

        // 解析得到 header 的内容
        position = position + 4;
        let mut header_body_bytes = [0u8; 4];
        header_body_bytes.copy_from_slice(&src[position..(position + header_len)]);

        // 解析得到 body 的长度
        position = position + header_len;
        let mut body_len_bytes = [0u8; 4];
        body_len_bytes.copy_from_slice(&src[position..(position + 4)]);
        let body_len = u32::from_be_bytes(body_len_bytes) as usize;

        println!("decode body_len={}", body_len);
        if body_len == 0 {
            return Err(Error::InvalidProtocol);
        }

        // 解析得到 body 的内容
        position = position + 4;
        let mut body_bytes: [u8; 4] = [0u8; 4];
        body_bytes.copy_from_slice(&src[position..(position + body_len)]);

        println!("decode header_len={}", header_len);
        println!("decode body_len={}", body_len);

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
                return Err(Error::InvalidProtocol);
            }
        }
        Ok(None)
    }
}

fn produce_req(body_bytes: [u8; 4], header: Header) -> Result<Option<StorageEnginePacket>, Error> {
    match ProduceReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::ProduceReq(ProduceReq {
                header: Some(header),
                body: Some(body),
            });
            return Ok(Some(item));
        }
        Err(e) => {
            return Err(Error::InvalidProtocol);
        }
    }
}

fn produce_resp(body_bytes: [u8; 4], header: Header) -> Result<Option<StorageEnginePacket>, Error> {
    match ProduceRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::ProduceResp(ProduceResp {
                header: Some(header),
                body: Some(body),
            });
            return Ok(Some(item));
        }
        Err(e) => {
            return Err(Error::InvalidProtocol);
        }
    }
}

fn fetch_req(body_bytes: [u8; 4], header: Header) -> Result<Option<StorageEnginePacket>, Error> {
    match FetchReqBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::FetchReq(FetchReq {
                header: Some(header),
                body: Some(body),
            });
            return Ok(Some(item));
        }
        Err(e) => {
            return Err(Error::InvalidProtocol);
        }
    }
}

fn fetch_resp(body_bytes: [u8; 4], header: Header) -> Result<Option<StorageEnginePacket>, Error> {
    match FetchRespBody::decode(body_bytes.as_ref()) {
        Ok(body) => {
            let item = StorageEnginePacket::FetchResp(FetchResp {
                header: Some(header),
                body: Some(body),
            });
            return Ok(Some(item));
        }
        Err(e) => {
            return Err(Error::InvalidProtocol);
        }
    }
}
