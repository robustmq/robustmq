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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct ETLRule {
    pub decode_rule: Option<ETLOperator>,
    pub ops_rule_list: Vec<ETLOperator>,
    pub encode_rule: Option<ETLOperator>,
}

impl ETLRule {
    pub fn is_empty(&self) -> bool {
        self.decode_rule.is_none() || self.ops_rule_list.is_empty() || self.encode_rule.is_none()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ETLOperator {
    Decode(DecodeDeleteParams),
    Encode(EncodeDeleteParams),
    Extract(ExtractRuleParams),
    Rename(RenameRuleParams),
    KeepOnly(KeepOnlyRuleParams),
    Set(FilterSetParams),
    Delete(FilterDeleteParams),
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct ExtractRuleParams {
    pub field_mapping: HashMap<String, String>,
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct RenameRuleParams {
    pub field_mapping: HashMap<String, String>,
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct KeepOnlyRuleParams {
    pub keys: Vec<String>,
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct FilterSetParams {
    pub rules: Vec<SetRule>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SetRule {
    pub target_field: String,
    pub value: SetValue,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SetValue {
    Const { value: Value },
    Now,
    Cel { expr: String },
}

impl Default for SetValue {
    fn default() -> Self {
        SetValue::Const { value: Value::Null }
    }
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct FilterDeleteParams {
    pub keys: Vec<String>,
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct DecodeDeleteParams {
    pub data_type: DataDecodeType,
    pub line_separator: Option<String>,
    pub token_separator: Option<String>,
    pub kv_separator: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub enum DataDecodeType {
    #[default]
    JsonObject,
    JsonArray,
    JsonLines,
    KeyValueLine,
    PositionalLine,
    Csv,
    PlainText,
    Bytes,
    Protobuf,
    Xml,
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct EncodeDeleteParams {
    pub data_type: DataEncodeType,
    pub line_separator: Option<String>,
    pub token_separator: Option<String>,
    pub kv_separator: Option<String>,
}

#[derive(Serialize, Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub enum DataEncodeType {
    #[default]
    JsonObject,
    JsonArray,
    JsonLines,
    KeyValueLine,
    PositionalLine,
    Csv,
    PlainText,
    Bytes,
}
