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

use common_base::tools::now_second;
use serde::{Deserialize, Serialize};

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
}

impl Record {
    pub fn build_byte(data: Vec<u8>) -> Self {
        Record {
            offset: None,
            key: "".to_string(),
            data,
            tags: Vec::new(),
            timestamp: now_second(),
            header: Vec::new(),
        }
    }

    pub fn build_str(data: String) -> Self {
        Record {
            offset: None,
            key: "".to_string(),
            data: data.as_bytes().to_vec(),
            tags: Vec::new(),
            timestamp: now_second(),
            header: Vec::new(),
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
}
