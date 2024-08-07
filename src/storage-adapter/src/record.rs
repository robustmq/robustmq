// Copyright 2023 RobustMQ Team
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


use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Record {
    pub offset: u128,
    pub header: Option<Vec<Header>>,
    pub key: Option<String>,
    pub data: Vec<u8>,
    pub create_time: Option<u128>,
}

impl Record {
    pub fn build_a(
        key: Option<String>,
        data: Vec<u8>,
        header: Option<Vec<Header>>,
        create_time: Option<u128>,
    ) -> Self {
        return Record {
            offset: 0,
            key,
            data,
            create_time,
            header,
        };
    }

    pub fn build_b(data: Vec<u8>) -> Self {
        return Record {
            offset: 0,
            key: None,
            data,
            create_time: None,
            header: None,
        };
    }

    pub fn build_c(key: String, data: Vec<u8>) -> Self {
        return Record {
            offset: 0,
            key: Some(key),
            data,
            create_time: None,
            header: None,
        };
    }

    pub fn build_d(key: String, header: Vec<Header>, data: Vec<u8>) -> Self {
        return Record {
            offset: 0,
            key: Some(key),
            data,
            create_time: None,
            header: Some(header),
        };
    }

    pub fn build_e(data: String) -> Self {
        return Record {
            offset: 0,
            key: None,
            data: data.as_bytes().to_vec(),
            create_time: None,
            header: None,
        };
    }
}


