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

use std::sync::atomic::{AtomicU64, Ordering};

use bytes::Bytes;
use common_base::utils::serialize;
use pulsar::{producer, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};

use crate::storage::record::StorageRecordProtocolData;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RecordHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AdapterWriteRecord {
    pub record_id: u64,
    pub topic: String,
    pub header: Option<Vec<RecordHeader>>,
    pub key: Option<String>,
    pub tags: Option<Vec<String>>,
    pub data: Bytes,
    pub protocol_data: Option<StorageRecordProtocolData>,
}

static PACKET_ID_GENERATOR: AtomicU64 = AtomicU64::new(0);

impl AdapterWriteRecord {
    pub fn new(topic: impl Into<String>, data: impl Into<Bytes>) -> Self {
        Self {
            record_id: PACKET_ID_GENERATOR.fetch_add(1, Ordering::Relaxed),
            topic: topic.into(),
            data: data.into(),
            ..Default::default()
        }
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_header(mut self, header: Vec<RecordHeader>) -> Self {
        self.header = Some(header);
        self
    }

    pub fn with_protocol_data(mut self, protocol_data: Option<StorageRecordProtocolData>) -> Self {
        self.protocol_data = protocol_data;
        self
    }

    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    pub fn tags(&self) -> &[String] {
        self.tags.as_deref().unwrap_or(&[])
    }

    pub fn header(&self) -> &[RecordHeader] {
        self.header.as_deref().unwrap_or(&[])
    }

    /// Approximate size of the record in bytes.
    pub fn size(&self) -> usize {
        let mut total = self.topic.len() + self.data.len();
        if let Some(ref key) = self.key {
            total += key.len();
        }
        if let Some(ref tags) = self.tags {
            total += tags.iter().map(|t| t.len()).sum::<usize>();
        }
        if let Some(ref headers) = self.header {
            total += headers
                .iter()
                .map(|h| h.name.len() + h.value.len())
                .sum::<usize>();
        }
        total
    }
}

impl SerializeMessage for AdapterWriteRecord {
    fn serialize_message(
        input: crate::adapter::adapter_record::AdapterWriteRecord,
    ) -> Result<producer::Message, PulsarError> {
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
    fn test_builder_chain() {
        let record = AdapterWriteRecord::new("test/topic", b"hello".as_ref())
            .with_key("my-key")
            .with_tags(vec!["tag1".to_string()])
            .with_header(vec![RecordHeader {
                name: "x-source".to_string(),
                value: "mqtt".to_string(),
            }]);

        assert_eq!(record.topic, "test/topic");
        assert_eq!(record.key(), Some("my-key"));
        assert_eq!(record.tags(), &["tag1"]);
        assert_eq!(record.header()[0].name, "x-source");
    }

    #[test]
    fn test_serialization() {
        let record = AdapterWriteRecord::new("test/topic", b"payload".as_ref()).with_key("k");

        let serialized = serialize::serialize(&record).expect("serialize failed");
        let decoded: AdapterWriteRecord =
            serialize::deserialize(&serialized).expect("deserialize failed");

        assert_eq!(decoded.topic, record.topic);
        assert_eq!(decoded.key, record.key);
        assert_eq!(decoded.data.as_ref(), record.data.as_ref());
    }
}
