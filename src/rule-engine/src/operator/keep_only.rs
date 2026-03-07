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
use metadata_struct::connector::rule::KeepOnlyRuleParams;
use serde_json::{Map, Value};

/// Keep only specified top-level keys in each record.
///
/// - `keys`: key list to retain.
/// - Keys not present in a record are ignored.
pub fn operator_keep_only(
    params: &KeepOnlyRuleParams,
    data: &Vec<Map<String, Value>>,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    if params.keys.is_empty() {
        return Ok(data.clone());
    }

    for key in &params.keys {
        if key.trim().is_empty() {
            return Err(CommonError::CommonError(
                "keep_only key cannot be empty".to_string(),
            ));
        }
    }

    let mut results = Vec::with_capacity(data.len());
    for record in data {
        let mut output = Map::new();
        for key in &params.keys {
            if let Some(value) = record.get(key) {
                output.insert(key.clone(), value.clone());
            }
        }
        results.push(output);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::operator_keep_only;
    use metadata_struct::connector::rule::KeepOnlyRuleParams;
    use serde_json::{Map, Value};

    fn make_record(items: &[(&str, Value)]) -> Map<String, Value> {
        let mut map = Map::new();
        for (k, v) in items {
            map.insert((*k).to_string(), v.clone());
        }
        map
    }

    #[test]
    fn operator_keep_only_ok() {
        let params = KeepOnlyRuleParams {
            keys: vec!["a".to_string(), "c".to_string(), "not_exists".to_string()],
        };
        let data = vec![make_record(&[
            ("a", Value::String("A".to_string())),
            ("b", Value::Number(2_i64.into())),
            ("c", Value::Bool(true)),
        ])];

        let result = operator_keep_only(&params, &data).unwrap();
        let row = &result[0];
        assert_eq!(row.len(), 2);
        assert_eq!(row.get("a").and_then(|v| v.as_str()), Some("A"));
        assert_eq!(row.get("c").and_then(|v| v.as_bool()), Some(true));
        assert!(row.get("b").is_none());
    }

    #[test]
    fn operator_keep_only_empty_key_err() {
        let params = KeepOnlyRuleParams {
            keys: vec!["".to_string()],
        };
        let data = vec![make_record(&[("a", Value::String("A".to_string()))])];
        let err = operator_keep_only(&params, &data).unwrap_err();
        assert!(err.to_string().contains("keep_only key cannot be empty"));
    }
}
