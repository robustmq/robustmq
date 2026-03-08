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
use metadata_struct::connector::rule::FilterDeleteParams;
use serde_json::{Map, Value};

/// Delete top-level fields by key names in each record.
///
/// - `keys`: key list to delete.
/// - If a key does not exist in a record, it is ignored.
pub fn operator_delete(
    params: &FilterDeleteParams,
    data: &Vec<Map<String, Value>>,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    if params.keys.is_empty() {
        return Ok(data.clone());
    }

    for key in &params.keys {
        if key.trim().is_empty() {
            return Err(CommonError::CommonError(
                "delete key cannot be empty".to_string(),
            ));
        }
    }

    let mut results = Vec::with_capacity(data.len());
    for record in data {
        let mut output = record.clone();
        for key in &params.keys {
            output.remove(key);
        }
        results.push(output);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::operator_delete;
    use metadata_struct::connector::rule::FilterDeleteParams;
    use serde_json::{Map, Value};

    fn make_record(items: &[(&str, Value)]) -> Map<String, Value> {
        let mut map = Map::new();
        for (k, v) in items {
            map.insert((*k).to_string(), v.clone());
        }
        map
    }

    #[test]
    fn operator_delete_ok() {
        let params = FilterDeleteParams {
            keys: vec!["a".to_string(), "c".to_string(), "not_exists".to_string()],
        };
        let data = vec![make_record(&[
            ("a", Value::String("A".to_string())),
            ("b", Value::Number(2_i64.into())),
            ("c", Value::Bool(true)),
        ])];

        let result = operator_delete(&params, &data).unwrap();
        let row = &result[0];
        assert!(row.get("a").is_none());
        assert!(row.get("c").is_none());
        assert_eq!(row.get("b").and_then(|v| v.as_i64()), Some(2));
    }

    #[test]
    fn operator_delete_empty_key_err() {
        let params = FilterDeleteParams {
            keys: vec!["".to_string()],
        };
        let data = vec![make_record(&[("a", Value::String("A".to_string()))])];
        let err = operator_delete(&params, &data).unwrap_err();
        assert!(err.to_string().contains("delete key cannot be empty"));
    }
}
