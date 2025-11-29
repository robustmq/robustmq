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
use common_base::{
    tools::now_second,
    utils::{crc::calc_crc32, serialize},
};
use pulsar::{producer, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};

use crate::adapter::{self, serde_bytes_wrapper};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub offset: Option<u64>,
    pub header: Option<Vec<Header>>,
    pub key: Option<String>,

    #[serde(with = "serde_bytes_wrapper")]
    pub data: Bytes,

    pub tags: Option<Vec<String>>,
    pub timestamp: u64,
    pub crc_num: u32,
}

impl Record {
    /// Create a Record from raw bytes (Vec<u8>)
    pub fn from_bytes(data: Vec<u8>) -> Self {
        let data = Bytes::from(data);
        let crc_num = calc_crc32(&data);
        Record {
            offset: None,
            key: None,
            data,
            tags: None,
            timestamp: now_second(),
            header: None,
            crc_num,
        }
    }

    /// Create a Record from Bytes (zero-copy)
    pub fn from_bytes_shared(data: Bytes) -> Self {
        let crc_num = calc_crc32(&data);
        Record {
            offset: None,
            key: None,
            data,
            tags: None,
            timestamp: now_second(),
            header: None,
            crc_num,
        }
    }

    /// Create a Record from a byte slice (copy)
    pub fn from_slice(data: &[u8]) -> Self {
        let data = Bytes::copy_from_slice(data);
        let crc_num = calc_crc32(&data);
        Record {
            offset: None,
            key: None,
            data,
            tags: None,
            timestamp: now_second(),
            header: None,
            crc_num,
        }
    }

    /// Create a Record from a string
    pub fn from_string(data: String) -> Self {
        let data = Bytes::from(data.into_bytes());
        let crc_num = calc_crc32(&data);
        Record {
            offset: None,
            key: None,
            data,
            tags: None,
            timestamp: now_second(),
            header: None,
            crc_num,
        }
    }

    /// Set key (chainable)
    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    /// Set tags (chainable)
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set header (chainable)
    pub fn with_header(mut self, header: Vec<Header>) -> Self {
        self.header = Some(header);
        self
    }

    /// Set timestamp (chainable)
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set tags (mutable)
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = Some(tags);
    }

    /// Set header (mutable)
    pub fn set_header(&mut self, headers: Vec<Header>) {
        self.header = Some(headers);
    }

    /// Set key (mutable)
    pub fn set_key(&mut self, key: String) {
        self.key = Some(key);
    }

    /// Verify CRC32 checksum
    pub fn crc32_check(&self) -> bool {
        let crc_num = calc_crc32(&self.data);
        crc_num == self.crc_num
    }

    /// Get key as string slice
    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    /// Get tags slice
    pub fn tags(&self) -> &[String] {
        self.tags.as_deref().unwrap_or(&[])
    }

    /// Get header slice
    pub fn header(&self) -> &[Header] {
        self.header.as_deref().unwrap_or(&[])
    }

    /// Calculate the approximate size of the Record in bytes
    /// This includes all fields: data, key, tags, headers, and metadata
    pub fn size(&self) -> usize {
        let mut total_size = 0;

        // Fixed-size fields
        total_size += std::mem::size_of::<Option<u64>>(); // offset
        total_size += std::mem::size_of::<u64>(); // timestamp
        total_size += std::mem::size_of::<u32>(); // crc_num

        // data (the main payload)
        total_size += self.data.len();

        // key
        if let Some(ref key) = self.key {
            total_size += key.len();
        }

        // tags
        if let Some(ref tags) = self.tags {
            for tag in tags {
                total_size += tag.len();
            }
        }

        // headers
        if let Some(ref headers) = self.header {
            for header in headers {
                total_size += header.name.len();
                total_size += header.value.len();
            }
        }

        total_size
    }
}

impl SerializeMessage for Record {
    fn serialize_message(input: adapter::record::Record) -> Result<producer::Message, PulsarError> {
        // Use bincode for better performance (3-5x faster than JSON)
        let payload =
            serialize::serialize(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_bincode_serialization() {
        let record = Record {
            offset: Some(123),
            header: Some(vec![Header {
                name: "test_header".to_string(),
                value: "test_value".to_string(),
            }]),
            key: Some("test_key".to_string()),
            data: b"test_data_12345".to_vec().into(),
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
            timestamp: 987654321,
            crc_num: 12345,
        };

        // Serialize
        let serialized = serialize::serialize(&record).expect("Failed to serialize");

        // Deserialize
        let deserialized: Record =
            serialize::deserialize(&serialized).expect("Failed to deserialize");

        // Verify
        assert_eq!(record.offset, deserialized.offset);
        assert_eq!(record.key, deserialized.key);
        assert_eq!(record.data.as_ref(), deserialized.data.as_ref());
        assert_eq!(record.tags, deserialized.tags);
        assert_eq!(record.timestamp, deserialized.timestamp);
        assert_eq!(record.crc_num, deserialized.crc_num);
    }

    #[test]
    fn test_record_with_none_fields() {
        let record = Record {
            offset: None,
            header: None,
            key: None,
            data: Bytes::from(vec![1, 2, 3]),
            tags: None,
            timestamp: 111,
            crc_num: 222,
        };

        println!("Original record: {:?}", record);

        let serialized = serialize::serialize(&record).expect("Failed to serialize");
        println!("Serialized bytes length: {}", serialized.len());
        println!(
            "Serialized bytes (first 50): {:?}",
            &serialized[..50.min(serialized.len())]
        );

        let deserialized: Record =
            serialize::deserialize(&serialized).expect("Failed to deserialize");
        println!("Deserialized record: {:?}", deserialized);

        assert_eq!(record.offset, deserialized.offset);
        assert_eq!(record.key, deserialized.key);
        assert_eq!(record.data.as_ref(), deserialized.data.as_ref());
        assert_eq!(record.tags, deserialized.tags);
    }
}
