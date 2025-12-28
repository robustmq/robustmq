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
use common_base::{tools::now_second, utils::serialize};
use pulsar::{producer, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdapterWriteRecordHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdapterWriteRecord {
    pub pkid: u64,
    pub header: Option<Vec<AdapterWriteRecordHeader>>,
    pub key: Option<String>,
    pub tags: Option<Vec<String>>,
    pub data: Bytes,
    pub timestamp: u64,
}

impl AdapterWriteRecord {
    /// Create a Record from raw bytes (Vec<u8>)
    pub fn from_bytes(data: Vec<u8>) -> Self {
        let data = Bytes::from(data);
        AdapterWriteRecord {
            pkid: 0,
            key: None,
            data,
            tags: None,
            timestamp: now_second(),
            header: None,
        }
    }

    /// Create a Record from Bytes (zero-copy)
    pub fn from_bytes_shared(data: Bytes) -> Self {
        AdapterWriteRecord {
            pkid: 0,
            data,
            key: None,
            tags: None,
            timestamp: now_second(),
            header: None,
        }
    }

    /// Create a Record from a byte slice (copy)
    pub fn from_slice(data: &[u8]) -> Self {
        let data = Bytes::copy_from_slice(data);
        AdapterWriteRecord {
            pkid: 0,
            data,
            key: None,
            tags: None,
            timestamp: now_second(),
            header: None,
        }
    }

    /// Create a Record from a string
    pub fn from_string(data: String) -> Self {
        let data = Bytes::from(data.into_bytes());
        AdapterWriteRecord {
            pkid: 0,
            key: None,
            tags: None,
            data,
            timestamp: now_second(),
            header: None,
        }
    }

    /// Set pkid (chainable)
    pub fn with_pkid(mut self, pkid: u64) -> Self {
        self.pkid = pkid;
        self
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
    pub fn with_header(mut self, header: Vec<AdapterWriteRecordHeader>) -> Self {
        self.header = Some(header);
        self
    }

    /// Set timestamp (chainable)
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set pkid (mutable)
    pub fn set_pkid(&mut self, pkid: u64) {
        self.pkid = pkid;
    }

    /// Set tags (mutable)
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = Some(tags);
    }

    /// Set header (mutable)
    pub fn set_header(&mut self, headers: Vec<AdapterWriteRecordHeader>) {
        self.header = Some(headers);
    }

    /// Set key (mutable)
    pub fn set_key(&mut self, key: String) {
        self.key = Some(key);
    }

    /// Set timestamp (mutable)
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
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
    pub fn header(&self) -> &[AdapterWriteRecordHeader] {
        self.header.as_deref().unwrap_or(&[])
    }

    /// Calculate the approximate size of the Record in bytes
    /// This includes all fields: data, key, tags, headers, and metadata
    pub fn size(&self) -> usize {
        let mut total_size = 0;

        // Fixed-size fields
        total_size += std::mem::size_of::<u64>(); // pkid
        total_size += std::mem::size_of::<u64>(); // timestamp

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

impl SerializeMessage for AdapterWriteRecord {
    fn serialize_message(
        input: storage::adapter_record::AdapterWriteRecord,
    ) -> Result<producer::Message, PulsarError> {
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
        let record = AdapterWriteRecord {
            pkid: 1,
            header: Some(vec![AdapterWriteRecordHeader {
                name: "test_header".to_string(),
                value: "test_value".to_string(),
            }]),
            key: Some("test_key".to_string()),
            data: b"test_data_12345".to_vec().into(),
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
            timestamp: 987654321,
        };

        // Serialize
        let serialized = serialize::serialize(&record).expect("Failed to serialize");

        // Deserialize
        let deserialized: AdapterWriteRecord =
            serialize::deserialize(&serialized).expect("Failed to deserialize");

        // Verify
        assert_eq!(record.pkid, deserialized.pkid);
        assert_eq!(record.key, deserialized.key);
        assert_eq!(record.data.as_ref(), deserialized.data.as_ref());
        assert_eq!(record.tags, deserialized.tags);
        assert_eq!(record.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_record_with_none_fields() {
        let record = AdapterWriteRecord {
            pkid: 1,
            header: None,
            key: None,
            data: Bytes::from(vec![1, 2, 3]),
            tags: None,
            timestamp: 111,
        };

        println!("Original record: {:?}", record);

        let serialized = serialize::serialize(&record).expect("Failed to serialize");
        println!("Serialized bytes length: {}", serialized.len());
        println!(
            "Serialized bytes (first 50): {:?}",
            &serialized[..50.min(serialized.len())]
        );

        let deserialized: AdapterWriteRecord =
            serialize::deserialize(&serialized).expect("Failed to deserialize");
        println!("Deserialized record: {:?}", deserialized);

        assert_eq!(record.pkid, deserialized.pkid);
        assert_eq!(record.key, deserialized.key);
        assert_eq!(record.data.as_ref(), deserialized.data.as_ref());
        assert_eq!(record.tags, deserialized.tags);
        assert_eq!(record.timestamp, deserialized.timestamp);
    }
}
