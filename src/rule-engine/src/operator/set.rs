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

use cel::{Context, Program};
use chrono::Utc;
use common_base::error::common::CommonError;
use metadata_struct::connector::rule::{FilterSetParams, SetValue};
use serde_json::{Map, Value};

/// Set or overwrite fields in each record.
///
/// Supported value sources:
/// - const: fixed JSON value
/// - now: current UTC time (RFC3339 string)
/// - cel: CEL expression evaluated against record variables
pub fn operator_set(
    params: &FilterSetParams,
    data: &Vec<Map<String, Value>>,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    if params.rules.is_empty() {
        return Ok(data.clone());
    }

    let mut compiled_cel = Vec::with_capacity(params.rules.len());
    for rule in &params.rules {
        if rule.target_field.trim().is_empty() {
            return Err(CommonError::CommonError(
                "set target field cannot be empty".to_string(),
            ));
        }
        match &rule.value {
            SetValue::Cel { expr } => {
                if expr.trim().is_empty() {
                    return Err(CommonError::CommonError(
                        "set cel expression cannot be empty".to_string(),
                    ));
                }
                let program = Program::compile(expr).map_err(|e| {
                    CommonError::CommonError(format!("failed to compile cel expression: {e}"))
                })?;
                compiled_cel.push(Some(program));
            }
            _ => compiled_cel.push(None),
        }
    }

    let mut results = Vec::with_capacity(data.len());
    for record in data {
        let mut output = record.clone();
        for (idx, rule) in params.rules.iter().enumerate() {
            let target = rule.target_field.clone();
            let value = match &rule.value {
                SetValue::Const { value } => value.clone(),
                SetValue::Now => Value::String(Utc::now().to_rfc3339()),
                SetValue::Cel { .. } => {
                    let program = compiled_cel[idx].as_ref().ok_or_else(|| {
                        CommonError::CommonError("missing compiled cel program".to_string())
                    })?;
                    eval_cel_to_json(program, &output)?
                }
            };
            output.insert(target, value);
        }
        results.push(output);
    }
    Ok(results)
}

fn eval_cel_to_json(program: &Program, record: &Map<String, Value>) -> Result<Value, CommonError> {
    let mut context = Context::default();
    let record_value = Value::Object(record.clone());

    let cel_record = cel::to_value(&record_value)
        .map_err(|e| CommonError::CommonError(format!("failed to convert record to cel: {e}")))?;
    context.add_variable("record", cel_record).map_err(|e| {
        CommonError::CommonError(format!("failed to add cel variable 'record': {e}"))
    })?;

    for (key, value) in record {
        let cel_value = cel::to_value(value).map_err(|e| {
            CommonError::CommonError(format!("failed to convert field '{key}' to cel: {e}"))
        })?;
        context.add_variable(key, cel_value).map_err(|e| {
            CommonError::CommonError(format!("failed to add cel variable '{key}': {e}"))
        })?;
    }

    let result = program
        .execute(&context)
        .map_err(|e| CommonError::CommonError(format!("failed to execute cel expression: {e}")))?;
    result
        .json()
        .map_err(|e| CommonError::CommonError(format!("failed to convert cel value to json: {e}")))
}

#[cfg(test)]
mod tests {
    use super::operator_set;
    use metadata_struct::connector::rule::{FilterSetParams, SetRule, SetValue};
    use serde_json::{json, Map, Value};

    fn make_record(items: &[(&str, Value)]) -> Map<String, Value> {
        let mut map = Map::new();
        for (k, v) in items {
            map.insert((*k).to_string(), v.clone());
        }
        map
    }

    #[test]
    fn operator_set_const_now_cel_ok() {
        let params = FilterSetParams {
            rules: vec![
                SetRule {
                    target_field: "tenant".to_string(),
                    value: SetValue::Const {
                        value: json!("tenant_a"),
                    },
                },
                SetRule {
                    target_field: "processed_at".to_string(),
                    value: SetValue::Now,
                },
                SetRule {
                    target_field: "temp_f".to_string(),
                    value: SetValue::Cel {
                        expr: "temp * 1.8 + 32.0".to_string(),
                    },
                },
            ],
        };

        let data = vec![make_record(&[
            ("device", json!("d1")),
            ("temp", json!(36.5)),
            ("count", json!(2)),
        ])];

        let result = operator_set(&params, &data).unwrap();
        let row = &result[0];
        assert_eq!(row.get("tenant"), Some(&json!("tenant_a")));
        assert!(row
            .get("processed_at")
            .and_then(|v| v.as_str())
            .map(|v| !v.is_empty())
            .unwrap_or(false));
        let temp_f = row.get("temp_f").and_then(|v| v.as_f64()).unwrap();
        assert!((temp_f - 97.7).abs() < 1e-9);
    }

    #[test]
    fn operator_set_empty_target_err() {
        let params = FilterSetParams {
            rules: vec![SetRule {
                target_field: "".to_string(),
                value: SetValue::Const { value: json!(1) },
            }],
        };
        let data = vec![make_record(&[("a", json!(1))])];
        let err = operator_set(&params, &data).unwrap_err();
        assert!(err.to_string().contains("set target field cannot be empty"));
    }
}
