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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct ETLRule {
    pub decode_rule: Option<ETLOperator>,
    pub ops_rule_list: Vec<ETLOperator>,
    pub encode_rule: Option<ETLOperator>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ETLOperator {
    Decode(DecodeDeleteParams),
    Filter(FilterRuleParams),
    Set(FilterSetParams),
    Delete(FilterDeleteParams),
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct FilterRuleParams {}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct FilterSetParams {}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct FilterDeleteParams {}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
pub struct DecodeDeleteParams {}
