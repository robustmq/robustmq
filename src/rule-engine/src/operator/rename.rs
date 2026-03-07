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
use metadata_struct::connector::rule::RenameRuleParams;
use serde_json::{Map, Value};

/// Rename top-level fields in each record using `field_mapping`.
///
/// - key: source field name
/// - value: target field name
///
/// If source field does not exist in a record, it is skipped.
pub fn operator_rename(
    params: &RenameRuleParams,
    data: &Vec<Map<String, Value>>,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    if params.field_mapping.is_empty() {
        return Ok(data.clone());
    }

    let mut results = Vec::with_capacity(data.len());
    for record in data {
        let mut output = record.clone();
        for (source_field, target_field) in &params.field_mapping {
            if source_field.trim().is_empty() {
                return Err(CommonError::CommonError(
                    "rename source field cannot be empty".to_string(),
                ));
            }
            if target_field.trim().is_empty() {
                return Err(CommonError::CommonError(
                    "rename target field cannot be empty".to_string(),
                ));
            }
            if source_field == target_field {
                continue;
            }

            if let Some(value) = output.remove(source_field) {
                output.insert(target_field.clone(), value);
            }
        }
        results.push(output);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::operator_rename;
    use metadata_struct::connector::rule::RenameRuleParams;
    use serde_json::{Map, Value};
    use std::collections::HashMap;

    fn make_record(items: &[(&str, Value)]) -> Map<String, Value> {
        let mut map = Map::new();
        for (k, v) in items {
            map.insert((*k).to_string(), v.clone());
        }
        map
    }

    #[test]
    fn operator_rename_ok() {
        let mut field_mapping = HashMap::new();
        field_mapping.insert("old_a".to_string(), "new_a".to_string());
        field_mapping.insert("old_b".to_string(), "new_b".to_string());
        field_mapping.insert("not_exists".to_string(), "unused".to_string());

        let params = RenameRuleParams { field_mapping };
        let data = vec![make_record(&[
            ("old_a", Value::String("A".to_string())),
            ("old_b", Value::Number(1_i64.into())),
            ("keep", Value::Bool(true)),
        ])];

        let result = operator_rename(&params, &data).unwrap();
        let row = &result[0];
        assert_eq!(row.get("new_a").and_then(|v| v.as_str()), Some("A"));
        assert_eq!(row.get("new_b").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(row.get("keep").and_then(|v| v.as_bool()), Some(true));
        assert!(row.get("old_a").is_none());
        assert!(row.get("old_b").is_none());
        assert!(row.get("unused").is_none());
    }

    #[test]
    fn operator_rename_empty_target_err() {
        let mut field_mapping = HashMap::new();
        field_mapping.insert("old_a".to_string(), "".to_string());
        let params = RenameRuleParams { field_mapping };
        let data = vec![make_record(&[("old_a", Value::String("A".to_string()))])];
        let err = operator_rename(&params, &data).unwrap_err();
        assert!(err.to_string().contains("target field cannot be empty"));
    }
}
