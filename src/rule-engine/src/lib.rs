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
use common_base::error::common::CommonError;
use metadata_struct::connector::rule::{ETLOperator, ETLRule};

pub mod decode;
pub mod encode;
pub mod rule_trait;

pub async fn apply_rule_engine(etl_rule: &ETLRule, data: &Bytes) -> Result<Bytes, CommonError> {
    if etl_rule.ops_rule_list.is_empty() {
        return Ok(data.clone());
    }

    for rule in etl_rule.ops_rule_list.iter() {
        match rule {
            ETLOperator::Decode(_params) => {}
            ETLOperator::Filter(_params) => {}
            ETLOperator::Set(_params) => {}
            ETLOperator::Delete(_params) => {}
        }
    }
    Ok(data.clone())
}
