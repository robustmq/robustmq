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

use common_base::{tools::now_second, utils::crc::calc_crc32};
use pulsar::{producer, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};

use crate::adapter;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub offset: Option<u64>,
    pub header: Vec<Header>,
    pub key: String,
    pub data: Vec<u8>,
    pub tags: Vec<String>,
    pub timestamp: u64,
    pub crc_num: u32,
}

impl Record {
    pub fn build_byte(data: Vec<u8>) -> Self {
        let crc_num = calc_crc32(&data);
        Record {
            offset: None,
            key: "".to_string(),
            data,
            tags: Vec::new(),
            timestamp: now_second(),
            header: Vec::new(),
            crc_num,
        }
    }

    pub fn build_str(data: String) -> Self {
        let crc_num = calc_crc32(data.as_bytes());
        let data = serde_json::to_vec(&data).unwrap();
        Record {
            offset: None,
            key: "".to_string(),
            data,
            tags: Vec::new(),
            timestamp: now_second(),
            header: Vec::new(),
            crc_num,
        }
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
    }

    pub fn set_header(&mut self, headers: Vec<Header>) {
        self.header = headers;
    }

    pub fn set_key(&mut self, key: String) {
        self.key = key;
    }

    pub fn crc32_check(&self) -> bool {
        let crc_num = calc_crc32(&self.data);
        crc_num == self.crc_num
    }
}

impl SerializeMessage for Record {
    fn serialize_message(input: adapter::record::Record) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}
