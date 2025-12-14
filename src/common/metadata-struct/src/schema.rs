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

use std::fmt::Display;

use common_base::{error::common::CommonError, utils::serialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SchemaData {
    pub name: String,
    pub schema_type: SchemaType,
    pub desc: String,
    pub schema: String,
}

impl SchemaData {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SchemaResourceBind {
    pub schema_name: String,
    pub resource_name: String,
}

impl SchemaResourceBind {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum SchemaType {
    JSON,
    PROTOBUF,
    AVRO,
}

impl Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaType::JSON => write!(f, "json"),
            SchemaType::PROTOBUF => write!(f, "protobuf"),
            SchemaType::AVRO => write!(f, "avro"),
        }
    }
}
