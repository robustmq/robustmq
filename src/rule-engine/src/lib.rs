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

#![allow(clippy::result_large_err)]
use bytes::Bytes;
use common_base::error::common::CommonError;
use metadata_struct::connector::rule::{ETLOperator, ETLRule};
use operator::extract::operator_extract;

use crate::{decode::operator_decode_data, encode::operator_encode_data};

pub mod decode;
pub mod encode;
pub mod operator;
pub mod rule_trait;
#[cfg(test)]
pub mod test_data;

pub async fn apply_rule_engine(etl_rule: &ETLRule, data: &Bytes) -> Result<Bytes, CommonError> {
    if etl_rule.is_empty() {
        return Ok(data.clone());
    }

    let decode_operator = etl_rule.decode_rule.clone().unwrap();

    let mut record_data = operator_decode_data(&decode_operator, data)?;
    for rule in etl_rule.ops_rule_list.iter() {
        match rule {
            ETLOperator::Decode(_) | ETLOperator::Encode(_) => {
                continue;
            }
            ETLOperator::Extract(params) => {
                record_data = operator_extract(params, &record_data)?;
            }
            ETLOperator::Set(_params) => {}
            ETLOperator::Delete(_params) => {}
        }
    }

    let encode_operator = etl_rule.encode_rule.clone().unwrap();
    let result = operator_encode_data(&encode_operator, record_data)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::apply_rule_engine;
    use metadata_struct::connector::rule::{
        DataDecodeType, DataEncodeType, DecodeDeleteParams, ETLOperator, ETLRule,
        EncodeDeleteParams, ExtractRuleParams,
    };
    use serde_json::Value;
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll, Waker};

    fn block_on<F: Future>(future: F) -> F::Output {
        let mut future = Pin::from(Box::new(future));
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[test]
    fn apply_rule_engine_extract_chain_ok() {
        let source = crate::test_data::gateway_source_json_bytes();

        let mut field_mapping = HashMap::new();
        field_mapping.insert("session.mqtt.topic".to_string(), "mqtt_topic".to_string());
        field_mapping.insert("gateway.network.wan.ip".to_string(), "wan_ip".to_string());
        field_mapping.insert(
            "/payload/alarms/0/active".to_string(),
            "alarm_active".to_string(),
        );
        field_mapping.insert("not.exists.path".to_string(), "missing".to_string());

        let etl_rule = ETLRule {
            decode_rule: Some(ETLOperator::Decode(DecodeDeleteParams {
                data_type: DataDecodeType::JsonObject,
                line_separator: None,
                token_separator: None,
                kv_separator: None,
            })),
            ops_rule_list: vec![ETLOperator::Extract(ExtractRuleParams { field_mapping })],
            encode_rule: Some(ETLOperator::Encode(EncodeDeleteParams {
                data_type: DataEncodeType::JsonObject,
                line_separator: None,
                token_separator: None,
                kv_separator: None,
            })),
        };

        println!(
            "etl_rule: {}",
            serde_json::to_string_pretty(&etl_rule).unwrap()
        );
        let result = block_on(apply_rule_engine(&etl_rule, &source)).unwrap();
        let output: Value = serde_json::from_slice(&result).unwrap();
        println!("output: {}", output);

        assert_eq!(
            output.get("mqtt_topic").and_then(|v| v.as_str()),
            Some("factory/a/line3/meter/44001/data")
        );
        assert_eq!(
            output.get("wan_ip").and_then(|v| v.as_str()),
            Some("10.8.12.34")
        );
        assert_eq!(
            output.get("alarm_active").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(output.get("missing").and_then(|v| v.as_str()), Some("-"));
        assert!(output.get("ts").is_none());
    }
}
