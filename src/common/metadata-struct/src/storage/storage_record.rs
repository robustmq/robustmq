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
use common_base::{tools::now_second, utils::crc::calc_crc32};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, Serialize, Deserialize)]
pub struct StorageRecordMetadata {
    pub offset: u64,
    pub shard: String,
    pub segment: u32,
    pub header: Option<Vec<Header>>,
    pub key: Option<String>,
    pub tags: Option<Vec<String>>,
    pub create_t: u64,
    pub crc_num: u32,
}

impl StorageRecordMetadata {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .unwrap()
            .to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)?)
    }

    pub fn new(
        offset: u64,
        shard: &str,
        segment: u32,
        header: &Option<Vec<Header>>,
        key: &Option<String>,
        tags: &Option<Vec<String>>,
        data: &Bytes,
    ) -> Self {
        StorageRecordMetadata {
            offset,
            shard: shard.to_string(),
            segment,
            header: header.clone(),
            key: key.clone(),
            tags: tags.clone(),
            create_t: now_second(),
            crc_num: calc_crc32(data),
        }
    }

    /// Create a minimal metadata (Builder pattern entry point)
    pub fn build(offset: u64, shard: String, segment: u32) -> Self {
        StorageRecordMetadata {
            offset,
            shard,
            segment,
            header: None,
            key: None,
            tags: None,
            create_t: now_second(),
            crc_num: 0,
        }
    }

    /// Set offset (chainable)
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    /// Set shard (chainable)
    pub fn with_shard(mut self, shard: String) -> Self {
        self.shard = shard;
        self
    }

    /// Set segment (chainable)
    pub fn with_segment(mut self, segment: u32) -> Self {
        self.segment = segment;
        self
    }

    /// Set header (chainable)
    pub fn with_header(mut self, header: Option<Vec<Header>>) -> Self {
        self.header = header;
        self
    }

    /// Set key (chainable)
    pub fn with_key(mut self, key: Option<String>) -> Self {
        self.key = key;
        self
    }

    /// Set tags (chainable)
    pub fn with_tags(mut self, tags: Option<Vec<String>>) -> Self {
        self.tags = tags;
        self
    }

    /// Set timestamp (chainable)
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.create_t = timestamp;
        self
    }

    /// Set create_t (chainable)
    pub fn with_create_t(mut self, create_t: u64) -> Self {
        self.create_t = create_t;
        self
    }

    /// Set crc_num from data (chainable)
    pub fn with_crc_from_data(mut self, data: &Bytes) -> Self {
        self.crc_num = calc_crc32(data);
        self
    }

    /// Set crc_num directly (chainable)
    pub fn with_crc_num(mut self, crc_num: u32) -> Self {
        self.crc_num = crc_num;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRecord {
    pub metadata: StorageRecordMetadata,
    pub data: Bytes,
}
