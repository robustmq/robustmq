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

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum ApiKey {
    Unimplemented,
    Read,
    Write,
}

impl Default for ApiKey {
    fn default() -> Self {
        Self::Unimplemented
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum ReadType {
    Offset,
    Key,
    Tag,
}

impl Default for ReadType {
    fn default() -> Self {
        Self::Offset
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct StorageEngineNetworkError {
    pub code: String,
    pub error: String,
}

impl StorageEngineNetworkError {
    pub fn new(code: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            error: error.into(),
        }
    }

    pub fn to_str(&self) -> String {
        format!("{}:{}", self.code, self.error)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ReqHeader {
    pub api_key: ApiKey,
}

impl ReqHeader {
    pub fn new(api_key: ApiKey) -> Self {
        Self { api_key }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct RespHeader {
    pub api_key: ApiKey,
    pub error: Option<StorageEngineNetworkError>,
}

impl RespHeader {
    pub fn new(api_key: ApiKey) -> Self {
        Self {
            api_key,
            error: None,
        }
    }

    pub fn with_error(api_key: ApiKey, error: StorageEngineNetworkError) -> Self {
        Self {
            api_key,
            error: Some(error),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct WriteReqBody {
    pub shard_name: String,
    pub messages: Vec<Vec<u8>>,
}

impl WriteReqBody {
    pub fn new(shard_name: String, messages: Vec<Vec<u8>>) -> Self {
        Self {
            shard_name,
            messages,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct WriteReq {
    pub header: ReqHeader,
    pub body: WriteReqBody,
}

impl WriteReq {
    pub fn new(body: WriteReqBody) -> Self {
        Self {
            header: ReqHeader::new(ApiKey::Write),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct WriteRespMessageStatus {
    pub offset: u64,
    pub pkid: u64,
    pub error: Option<StorageEngineNetworkError>,
}

impl WriteRespMessageStatus {
    pub fn new(offset: u64, pkid: u64) -> Self {
        Self {
            offset,
            pkid,
            error: None,
        }
    }

    pub fn with_error(offset: u64, pkid: u64, error: StorageEngineNetworkError) -> Self {
        Self {
            offset,
            pkid,
            error: Some(error),
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct WriteRespMessage {
    pub shard_name: String,
    pub messages: Vec<WriteRespMessageStatus>,
}

impl WriteRespMessage {
    pub fn new(shard_name: String, messages: Vec<WriteRespMessageStatus>) -> Self {
        Self {
            shard_name,
            messages,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct WriteRespBody {
    pub status: Vec<WriteRespMessage>,
}

impl WriteRespBody {
    pub fn new(status: Vec<WriteRespMessage>) -> Self {
        Self { status }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct WriteResp {
    pub header: RespHeader,
    pub body: WriteRespBody,
}

impl WriteResp {
    pub fn new(body: WriteRespBody) -> Self {
        Self {
            header: RespHeader::new(ApiKey::Write),
            body,
        }
    }

    pub fn with_error(error: StorageEngineNetworkError) -> Self {
        Self {
            header: RespHeader::with_error(ApiKey::Write, error),
            body: WriteRespBody::default(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ReadReqFilter {
    pub timestamp: Option<u64>,
    pub offset: Option<u64>,
    pub key: Option<String>,
    pub tag: Option<String>,
}

impl ReadReqFilter {
    pub fn by_offset(offset: u64) -> Self {
        Self {
            offset: Some(offset),
            ..Default::default()
        }
    }

    pub fn by_key(key: String) -> Self {
        Self {
            key: Some(key),
            ..Default::default()
        }
    }

    pub fn by_tag(tag: String) -> Self {
        Self {
            tag: Some(tag),
            ..Default::default()
        }
    }

    pub fn by_timestamp(timestamp: u64) -> Self {
        Self {
            timestamp: Some(timestamp),
            ..Default::default()
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq)]
pub struct ReadReqOptions {
    pub max_size: u64,
    pub max_record: u64,
}

impl Default for ReadReqOptions {
    fn default() -> Self {
        Self {
            max_size: 1024 * 1024,
            max_record: 100,
        }
    }
}

impl ReadReqOptions {
    pub fn new(max_size: u64, max_record: u64) -> Self {
        Self {
            max_size,
            max_record,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ReadReqMessage {
    pub shard_name: String,
    pub read_type: ReadType,
    pub filter: ReadReqFilter,
    pub options: ReadReqOptions,
}

impl ReadReqMessage {
    pub fn new(
        shard_name: String,
        read_type: ReadType,
        filter: ReadReqFilter,
        options: ReadReqOptions,
    ) -> Self {
        Self {
            shard_name,
            read_type,
            filter,
            options,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ReadReqBody {
    pub messages: Vec<ReadReqMessage>,
}

impl ReadReqBody {
    pub fn new(messages: Vec<ReadReqMessage>) -> Self {
        Self { messages }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ReadReq {
    pub header: ReqHeader,
    pub body: ReadReqBody,
}

impl ReadReq {
    pub fn new(body: ReadReqBody) -> Self {
        Self {
            header: ReqHeader::new(ApiKey::Read),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ReadRespBody {
    pub messages: Vec<Vec<u8>>,
}

impl ReadRespBody {
    pub fn new(messages: Vec<Vec<u8>>) -> Self {
        Self { messages }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ReadResp {
    pub header: RespHeader,
    pub body: ReadRespBody,
}

impl ReadResp {
    pub fn new(body: ReadRespBody) -> Self {
        Self {
            header: RespHeader::new(ApiKey::Read),
            body,
        }
    }

    pub fn with_error(error: StorageEngineNetworkError) -> Self {
        Self {
            header: RespHeader::with_error(ApiKey::Read, error),
            body: ReadRespBody::default(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_request_encode_decode() {
        let filter = ReadReqFilter::by_offset(100);
        let options = ReadReqOptions::new(1024 * 1024, 100);
        let message = ReadReqMessage::new("shard1".to_string(), ReadType::Offset, filter, options);
        let body = ReadReqBody::new(vec![message]);
        let req = ReadReq::new(body);

        let encoded = req.encode();
        let decoded = ReadReq::decode(&encoded).unwrap();

        assert_eq!(decoded.body.messages.len(), 1);
        assert_eq!(decoded.body.messages[0].shard_name, "shard1");
        assert_eq!(decoded.body.messages[0].filter.offset.unwrap(), 100);
    }
}
