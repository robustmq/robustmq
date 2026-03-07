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

use common_base::error::common::CommonError;
use metadata_struct::connector::rule::ExtractRuleParams;
use serde_json::{Map, Value};

/// Extract fields from each record based on `field_mapping`.
///
/// Config demo:
/// {
///   "field_mapping": {
///     "session.mqtt.topic": "mqtt_topic",
///     "gateway.network.wan.ip": "wan_ip",
///     "/payload/alarms/0/active": "alarm_active",
///     "not.exists.path": "missing"
///   }
/// }
///
/// Input demo (single row):
/// {
///   "session": { "mqtt": { "topic": "factory/a/line3/meter/44001/data" } },
///   "gateway": { "network": { "wan": { "ip": "10.8.12.34" } } },
///   "payload": { "alarms": [ { "active": true } ] }
/// }
///
/// Output demo (single row):
/// {
///   "mqtt_topic": "factory/a/line3/meter/44001/data",
///   "wan_ip": "10.8.12.34",
///   "alarm_active": true,
///   "missing": "-"
/// }
///
/// Notes:
/// - source supports both dot path and JSON Pointer path.
/// - missing source field is filled with "-".
pub fn operator_extract(
    params: &ExtractRuleParams,
    data: &Vec<Map<String, Value>>,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    if params.field_mapping.is_empty() {
        return Ok(data.clone());
    }

    let mut results = Vec::with_capacity(data.len());
    for record in data {
        let mut output = Map::with_capacity(params.field_mapping.len());
        let root = Value::Object(record.clone());

        for (source_field, target_field) in &params.field_mapping {
            if target_field.trim().is_empty() {
                return Err(CommonError::CommonError(
                    "extract target field cannot be empty".to_string(),
                ));
            }

            let pointer = field_path_to_pointer(source_field);
            let value = root
                .pointer(&pointer)
                .cloned()
                .unwrap_or_else(|| Value::String("-".to_string()));
            output.insert(target_field.clone(), value);
        }
        results.push(output);
    }

    Ok(results)
}

fn field_path_to_pointer(path: &str) -> String {
    if path.starts_with('/') {
        return path.to_string();
    }
    let segments = path
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(escape_json_pointer_segment)
        .collect::<Vec<_>>();
    format!("/{}", segments.join("/"))
}

fn escape_json_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::operator_extract;
    use crate::test_data::gateway_source_json_map;
    use metadata_struct::connector::rule::ExtractRuleParams;
    use std::collections::HashMap;

    #[test]
    fn operator_extract_complex_mapping_ok() {
        let mut field_mapping = HashMap::new();
        field_mapping.insert("session.mqtt.topic".to_string(), "mqtt_topic".to_string());
        field_mapping.insert("gateway.network.wan.ip".to_string(), "wan_ip".to_string());
        field_mapping.insert(
            "/payload/alarms/0/active".to_string(),
            "alarm_active".to_string(),
        );
        field_mapping.insert(
            "payload.metrics.2.value".to_string(),
            "temperature".to_string(),
        );
        field_mapping.insert("not.exists.path".to_string(), "missing".to_string());

        let params = ExtractRuleParams { field_mapping };
        let data = vec![gateway_source_json_map()];
        let result = operator_extract(&params, &data).unwrap();
        let row = &result[0];

        assert_eq!(
            row.get("mqtt_topic").and_then(|v| v.as_str()),
            Some("factory/a/line3/meter/44001/data")
        );
        assert_eq!(
            row.get("wan_ip").and_then(|v| v.as_str()),
            Some("10.8.12.34")
        );
        assert_eq!(
            row.get("alarm_active").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(row.get("temperature").and_then(|v| v.as_f64()), Some(36.5));
        assert_eq!(row.get("missing").and_then(|v| v.as_str()), Some("-"));
        assert!(row.get("ts").is_none());
    }

    #[test]
    fn operator_extract_empty_target_field_err() {
        let mut field_mapping = HashMap::new();
        field_mapping.insert("session.mqtt.topic".to_string(), "".to_string());
        let params = ExtractRuleParams { field_mapping };
        let data = vec![gateway_source_json_map()];
        let err = operator_extract(&params, &data).unwrap_err();
        assert!(err.to_string().contains("target field cannot be empty"));
    }
}
