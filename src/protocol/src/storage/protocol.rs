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

use bytes::Bytes;
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
    Fetch,
    OffsetsForLeaderEpoch,
    ShardOffset,
    Delete,
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

/// Default acks=all commit timeout (ms) when the producer doesn't specify one.
pub const DEFAULT_WRITE_TIMEOUT_MS: u64 = 30_000;

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct WriteReqBody {
    pub shard_name: String,
    pub messages: Vec<Vec<u8>>,
    pub acks: i8,
    pub current_leader_epoch: u32,
    pub timeout_ms: u64,
}

impl WriteReqBody {
    pub fn new(shard_name: String, messages: Vec<Vec<u8>>) -> Self {
        Self {
            shard_name,
            messages,
            acks: 1,
            current_leader_epoch: 0,
            timeout_ms: DEFAULT_WRITE_TIMEOUT_MS,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ReadReqFilter {
    pub timestamp: Option<u64>,
    pub offset: Option<u64>,
    pub key: Option<Bytes>,
    pub tag: Option<String>,
}

impl ReadReqFilter {
    pub fn by_offset(offset: u64) -> Self {
        Self {
            offset: Some(offset),
            ..Default::default()
        }
    }

    pub fn by_key(key: impl Into<Bytes>) -> Self {
        Self {
            key: Some(key.into()),
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
    pub batch_call_source: bool,
    pub filter: ReadReqFilter,
    pub options: ReadReqOptions,
}

impl ReadReqMessage {
    pub fn new(
        shard_name: String,
        read_type: ReadType,
        batch_call_source: bool,
        filter: ReadReqFilter,
        options: ReadReqOptions,
    ) -> Self {
        Self {
            shard_name,
            read_type,
            batch_call_source,
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

// ISR follower fetch (isr.md §6.7). Batched across shards: a broker follows
// thousands of shards, each pulling its one active segment.

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct FetchShardReq {
    pub shard_name: String,
    pub segment_seq: u32,
    pub fetch_offset: u64,
    pub current_leader_epoch: u32,
    pub max_bytes: u64,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct FetchReqBody {
    pub replica_id: u64,
    pub replica_broker_epoch: u64,
    pub min_bytes: u64,
    pub max_wait_ms: u64,
    pub shards: Vec<FetchShardReq>,
}

impl FetchReqBody {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct FetchReq {
    pub header: ReqHeader,
    pub body: FetchReqBody,
}

impl FetchReq {
    pub fn new(body: FetchReqBody) -> Self {
        Self {
            header: ReqHeader::new(ApiKey::Fetch),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct FetchShardResp {
    pub shard_name: String,
    pub segment_seq: u32,
    pub records: Vec<Vec<u8>>,
    pub leader_hw: u64,
    pub leader_log_start: u64,
    pub leader_leo: u64,
    pub leader_epoch: u32,
    pub error_code: u32,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct FetchRespBody {
    pub shards: Vec<FetchShardResp>,
}

impl FetchRespBody {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct FetchResp {
    pub header: RespHeader,
    pub body: FetchRespBody,
}

impl FetchResp {
    pub fn new(body: FetchRespBody) -> Self {
        Self {
            header: RespHeader::new(ApiKey::Fetch),
            body,
        }
    }

    pub fn with_error(error: StorageEngineNetworkError) -> Self {
        Self {
            header: RespHeader::with_error(ApiKey::Fetch, error),
            body: FetchRespBody::default(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

/// Per-shard fetch result code (isr.md §6.2); rejections are normal under
/// leader change / retention.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchErrorCode {
    None = 0,
    NotLeaderForPartition = 1,
    FencedLeaderEpoch = 2,
    UnknownLeaderEpoch = 3,
    OffsetOutOfRange = 4,
    StaleBrokerEpoch = 5,
}

impl FetchErrorCode {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct OffsetsForLeaderEpochReqBody {
    pub shard_name: String,
    pub segment_seq: u32,
    pub replica_id: u64,
    pub replica_broker_epoch: u64,
    pub current_leader_epoch: u32,
    pub follower_leader_epoch: u32,
}

impl OffsetsForLeaderEpochReqBody {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq)]
pub struct OffsetsForLeaderEpochReq {
    pub header: ReqHeader,
    pub body: OffsetsForLeaderEpochReqBody,
}

impl OffsetsForLeaderEpochReq {
    pub fn new(body: OffsetsForLeaderEpochReqBody) -> Self {
        Self {
            header: ReqHeader::new(ApiKey::OffsetsForLeaderEpoch),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct OffsetsForLeaderEpochRespBody {
    pub truncate_epoch: i32,
    pub truncate_offset: u64,
    pub error_code: u32,
    pub current_leader_epoch: u32,
}

impl OffsetsForLeaderEpochRespBody {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct OffsetsForLeaderEpochResp {
    pub header: RespHeader,
    pub body: OffsetsForLeaderEpochRespBody,
}

impl OffsetsForLeaderEpochResp {
    pub fn new(body: OffsetsForLeaderEpochRespBody) -> Self {
        Self {
            header: RespHeader::new(ApiKey::OffsetsForLeaderEpoch),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

/// Query a shard's offsets from its leader. `by_timestamp=false` returns the
/// shard's current start/end offsets; `by_timestamp=true` resolves the offset of
/// the first record at/after `timestamp`. Used by consumers on non-leader nodes to
/// resolve a start offset against the node that actually holds the data.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ShardOffsetReqBody {
    pub shard_name: String,
    pub by_timestamp: bool,
    pub timestamp: u64,
    /// Fallback strategy when no exact timestamp match: 0=Earliest, 1=Latest.
    pub strategy: u8,
}

impl ShardOffsetReqBody {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq)]
pub struct ShardOffsetReq {
    pub header: ReqHeader,
    pub body: ShardOffsetReqBody,
}

impl ShardOffsetReq {
    pub fn new(body: ShardOffsetReqBody) -> Self {
        Self {
            header: ReqHeader::new(ApiKey::ShardOffset),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ShardOffsetRespBody {
    pub start_offset: u64,
    pub end_offset: u64,
    pub high_watermark: u64,
    /// Offset resolved by timestamp (valid when the request set `by_timestamp`).
    pub offset: u64,
    pub error_code: u32,
}

impl ShardOffsetRespBody {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ShardOffsetResp {
    pub header: RespHeader,
    pub body: ShardOffsetRespBody,
}

impl ShardOffsetResp {
    pub fn new(body: ShardOffsetRespBody) -> Self {
        Self {
            header: RespHeader::new(ApiKey::ShardOffset),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct DeleteReqBody {
    pub shard_name: String,
    pub keys: Vec<Bytes>,
    pub offsets: Vec<u64>,
    /// When set, delete all records with offset strictly less than this value
    /// (Kafka DeleteRecords semantics) instead of using `keys`/`offsets`.
    pub delete_before_offset: Option<u64>,
}

impl DeleteReqBody {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq)]
pub struct DeleteReq {
    pub header: ReqHeader,
    pub body: DeleteReqBody,
}

impl DeleteReq {
    pub fn new(body: DeleteReqBody) -> Self {
        Self {
            header: ReqHeader::new(ApiKey::Delete),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct DeleteRespBody {
    pub error_code: u32,
    /// The low_watermark achieved after a `delete_before_offset` request.
    pub achieved_offset: u64,
}

impl DeleteRespBody {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Default, PartialEq)]
pub struct DeleteResp {
    pub header: RespHeader,
    pub body: DeleteRespBody,
}

impl DeleteResp {
    pub fn new(body: DeleteRespBody) -> Self {
        Self {
            header: RespHeader::new(ApiKey::Delete),
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_request_encode_decode() {
        let filter = ReadReqFilter::by_offset(100);
        let options = ReadReqOptions::new(1024 * 1024, 100);
        let message = ReadReqMessage::new(
            "shard1".to_string(),
            ReadType::Offset,
            false,
            filter,
            options,
        );
        let body = ReadReqBody::new(vec![message]);
        let req = ReadReq::new(body);

        let encoded = req.encode();
        let decoded = ReadReq::decode(&encoded).unwrap();

        assert_eq!(decoded.body.messages.len(), 1);
        assert_eq!(decoded.body.messages[0].shard_name, "shard1");
        assert_eq!(decoded.body.messages[0].filter.offset.unwrap(), 100);
    }
}
