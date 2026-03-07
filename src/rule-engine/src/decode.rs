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
use metadata_struct::connector::rule::{DataDecodeType, ETLOperator};
use serde_json::{Map, Value};

/// Demo input: `{"device":"d1","temp":36.5}`
/// Demo output: `[{"device":"d1","temp":36.5}]`
pub fn operator_decode_data(
    etl_operator: &ETLOperator,
    data: &Bytes,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    let params = if let ETLOperator::Decode(params) = etl_operator {
        params
    } else {
        return Err(CommonError::CommonError(
            "operator_decode_data expects ETLOperator::Decode".to_string(),
        ));
    };

    match params.data_type {
        DataDecodeType::JsonObject => decode_json_object(data),
        DataDecodeType::JsonArray => decode_json_array(data),
        DataDecodeType::JsonLines => decode_json_lines(data, params.line_separator.as_deref()),
        DataDecodeType::KeyValueLine => decode_key_value_line(
            data,
            params.token_separator.as_deref(),
            params.kv_separator.as_deref(),
        ),
        DataDecodeType::PositionalLine => {
            decode_positional_line(data, params.token_separator.as_deref())
        }
        DataDecodeType::Csv => decode_csv(data),
        DataDecodeType::PlainText => decode_plain_text(data),
        DataDecodeType::Bytes => decode_raw_bytes(data),
        DataDecodeType::Protobuf => Err(CommonError::CommonError(
            "protobuf decode is not supported yet".to_string(),
        )),
        DataDecodeType::Xml => Err(CommonError::CommonError(
            "xml decode is not supported yet".to_string(),
        )),
    }
}

/// Demo input: `{"device":"d1","temp":36.5}`
/// Demo output: `[{"device":"d1","temp":36.5}]`
fn decode_json_object(data: &[u8]) -> Result<Vec<Map<String, Value>>, CommonError> {
    let value: Value = serde_json::from_slice(data).map_err(|e| {
        CommonError::CommonError(format!("failed to parse json object payload: {e}"))
    })?;
    match value {
        Value::Object(map) => Ok(vec![map]),
        _ => Err(CommonError::CommonError(
            "json_object decode expects root object".to_string(),
        )),
    }
}

/// Demo input: `[{"device":"d1"},{"device":"d2"}]`
/// Demo output: `[{"device":"d1"},{"device":"d2"}]`
fn decode_json_array(data: &[u8]) -> Result<Vec<Map<String, Value>>, CommonError> {
    let value: Value = serde_json::from_slice(data).map_err(|e| {
        CommonError::CommonError(format!("failed to parse json array payload: {e}"))
    })?;
    match value {
        Value::Array(items) => {
            let mut records = Vec::with_capacity(items.len());
            for (idx, item) in items.into_iter().enumerate() {
                match item {
                    Value::Object(map) => records.push(map),
                    _ => {
                        return Err(CommonError::CommonError(format!(
                            "json_array decode expects each item to be an object, index: {idx}"
                        )));
                    }
                }
            }
            Ok(records)
        }
        _ => Err(CommonError::CommonError(
            "json_array decode expects root array".to_string(),
        )),
    }
}

/// Demo input: `{"d":"d1"}\n{"d":"d2"}`
/// Demo output: `[{"d":"d1"},{"d":"d2"}]`
fn decode_json_lines(
    data: &[u8],
    line_separator: Option<&str>,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    let text = std::str::from_utf8(data).map_err(|e| {
        CommonError::CommonError(format!("failed to parse json lines as utf-8 text: {e}"))
    })?;

    let line_separator = line_separator.filter(|sep| !sep.is_empty());

    let mut records = Vec::new();
    let lines: Vec<&str> = if let Some(sep) = line_separator {
        text.split(sep).collect()
    } else {
        text.lines().collect()
    };
    for (idx, line) in lines.into_iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(trimmed).map_err(|e| {
            CommonError::CommonError(format!("failed to parse json line {}: {e}", idx + 1))
        })?;
        match value {
            Value::Object(map) => records.push(map),
            _ => {
                return Err(CommonError::CommonError(format!(
                    "json line {} is not an object",
                    idx + 1
                )));
            }
        }
    }
    Ok(records)
}

/// Demo input: `ts=2026-03-03 level=INFO temp=36.5`
/// Demo output: `[{"ts":"2026-03-03","level":"INFO","temp":36.5}]`
fn decode_key_value_line(
    data: &[u8],
    token_separator: Option<&str>,
    kv_separator: Option<&str>,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    let text = std::str::from_utf8(data).map_err(|e| {
        CommonError::CommonError(format!("failed to parse key=value line as utf-8 text: {e}"))
    })?;

    let token_separator = token_separator.filter(|sep| !sep.is_empty());
    let kv_separator = kv_separator.filter(|sep| !sep.is_empty());

    let mut map = Map::new();
    let tokens: Vec<&str> = if let Some(sep) = token_separator {
        text.split(sep).collect()
    } else {
        text.split_whitespace().collect()
    };
    let kv_sep = kv_separator.unwrap_or("=");

    for token in tokens {
        if token.is_empty() {
            continue;
        }
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let (key, raw_value) = token
            .split_once(kv_sep)
            .ok_or_else(|| CommonError::CommonError(format!("invalid key=value token: {token}")))?;
        if key.is_empty() {
            return Err(CommonError::CommonError(
                "key=value token has empty key".to_string(),
            ));
        }
        map.insert(key.to_string(), parse_scalar_value(raw_value));
    }
    Ok(vec![map])
}

/// Demo input: `2026-03-03 INFO d1 36.5`
/// Demo output: `[{"col_0":"2026-03-03","col_1":"INFO","col_2":"d1","col_3":36.5}]`
fn decode_positional_line(
    data: &[u8],
    token_separator: Option<&str>,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    let text = std::str::from_utf8(data).map_err(|e| {
        CommonError::CommonError(format!(
            "failed to parse positional line as utf-8 text: {e}"
        ))
    })?;

    let token_separator = token_separator.filter(|sep| !sep.is_empty());

    let mut map = Map::new();
    let tokens: Vec<&str> = if let Some(sep) = token_separator {
        text.split(sep).collect()
    } else {
        text.split_whitespace().collect()
    };
    for (idx, token) in tokens
        .into_iter()
        .filter(|v| !v.trim().is_empty())
        .enumerate()
    {
        let token = token.trim();
        map.insert(format!("col_{idx}"), parse_scalar_value(token));
    }
    if map.is_empty() {
        return Err(CommonError::CommonError(
            "positional_line decode got empty input".to_string(),
        ));
    }
    Ok(vec![map])
}

/// Demo input: `ts,level,temp\n2026-03-03,INFO,36.5`
/// Demo output: `[{"ts":"2026-03-03","level":"INFO","temp":36.5}]`
fn decode_csv(data: &[u8]) -> Result<Vec<Map<String, Value>>, CommonError> {
    let text = std::str::from_utf8(data)
        .map_err(|e| CommonError::CommonError(format!("failed to parse csv as utf-8 text: {e}")))?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| CommonError::CommonError("csv payload is empty".to_string()))?;
    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let mut records = Vec::new();
    for (idx, line) in lines.enumerate() {
        let cells: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if cells.len() != headers.len() {
            return Err(CommonError::CommonError(format!(
                "csv row {} has {} cells but {} headers",
                idx + 2,
                cells.len(),
                headers.len()
            )));
        }
        let mut map = Map::new();
        for (header, cell) in headers.iter().zip(cells.iter()) {
            map.insert(header.clone(), parse_scalar_value(cell));
        }
        records.push(map);
    }

    Ok(records)
}

/// Demo input: `device d1 temp 36.5`
/// Demo output: `[{"message":"device d1 temp 36.5"}]`
fn decode_plain_text(data: &[u8]) -> Result<Vec<Map<String, Value>>, CommonError> {
    let text = std::str::from_utf8(data).map_err(|e| {
        CommonError::CommonError(format!("failed to parse plain text as utf-8: {e}"))
    })?;
    let mut map = Map::new();
    map.insert("message".to_string(), Value::String(text.to_string()));
    Ok(vec![map])
}

/// Demo input: `[137,80,78,71]`
/// Demo output: `[{"bytes":[137,80,78,71]}]`
fn decode_raw_bytes(data: &[u8]) -> Result<Vec<Map<String, Value>>, CommonError> {
    let mut map = Map::new();
    let items = data
        .iter()
        .map(|v| Value::Number(serde_json::Number::from(*v)))
        .collect::<Vec<_>>();
    map.insert("bytes".to_string(), Value::Array(items));
    Ok(vec![map])
}

fn parse_scalar_value(raw: &str) -> Value {
    if raw.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if raw.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Ok(i) = raw.parse::<i64>() {
        return Value::Number(serde_json::Number::from(i));
    }
    if let Ok(f) = raw.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(f) {
            return Value::Number(num);
        }
    }
    Value::String(raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::{operator_decode_data, DataDecodeType};
    use bytes::Bytes;
    use metadata_struct::connector::rule::{DecodeDeleteParams, ETLOperator};
    use serde_json::Value;

    type ExpectedFieldPairs<'a> = Vec<(&'a str, Value)>;
    type DecodeCase<'a> = (ETLOperator, Bytes, ExpectedFieldPairs<'a>);

    fn build_decode_op(data_type: DataDecodeType) -> ETLOperator {
        ETLOperator::Decode(DecodeDeleteParams {
            data_type,
            line_separator: None,
            token_separator: None,
            kv_separator: None,
        })
    }

    fn build_decode_op_with_separators(
        data_type: DataDecodeType,
        line_separator: Option<&str>,
        token_separator: Option<&str>,
        kv_separator: Option<&str>,
    ) -> ETLOperator {
        ETLOperator::Decode(DecodeDeleteParams {
            data_type,
            line_separator: line_separator.map(|v| v.to_string()),
            token_separator: token_separator.map(|v| v.to_string()),
            kv_separator: kv_separator.map(|v| v.to_string()),
        })
    }

    #[test]
    fn decode_json_object_and_array_ok() {
        let data = Bytes::from(r#"{"k":"v","n":1}"#);
        let op = build_decode_op(DataDecodeType::JsonObject);
        let result = operator_decode_data(&op, &data).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("k").and_then(|v| v.as_str()), Some("v"));
        assert_eq!(result[0].get("n").and_then(|v| v.as_i64()), Some(1));
        let array_data = Bytes::from(r#"[{"k":"a"},{"k":"b"}]"#);
        let array_op = build_decode_op(DataDecodeType::JsonArray);
        let array_result = operator_decode_data(&array_op, &array_data).unwrap();
        assert_eq!(array_result.len(), 2);
    }

    #[test]
    fn decode_positional_line_empty_err() {
        let data = Bytes::from("   ");
        let op = build_decode_op(DataDecodeType::PositionalLine);
        let result = operator_decode_data(&op, &data);
        assert!(result.is_err());
    }

    #[test]
    fn decode_json_array_non_object_element_has_index() {
        let data = Bytes::from(r#"[{"k":"v"}, 42]"#);
        let op = build_decode_op(DataDecodeType::JsonArray);
        let err = operator_decode_data(&op, &data).unwrap_err();
        assert!(err.to_string().contains("index: 1"));
    }

    #[test]
    fn decode_with_separators_cases() {
        let cases: Vec<DecodeCase<'_>> = vec![
            (
                build_decode_op_with_separators(DataDecodeType::JsonLines, Some("||"), None, None),
                Bytes::from(r#"{"d":"d1"}||{"d":"d2"}"#),
                vec![("d", Value::String("d2".to_string()))],
            ),
            (
                build_decode_op_with_separators(
                    DataDecodeType::KeyValueLine,
                    None,
                    Some(";"),
                    Some(":"),
                ),
                Bytes::from("a:1;b:true;c:hello"),
                vec![
                    ("a", Value::Number(1_i64.into())),
                    ("b", Value::Bool(true)),
                    ("c", Value::String("hello".to_string())),
                ],
            ),
            (
                build_decode_op_with_separators(
                    DataDecodeType::KeyValueLine,
                    None,
                    Some(""),
                    Some(""),
                ),
                Bytes::from("a=1 b=true c=hello"),
                vec![
                    ("a", Value::Number(1_i64.into())),
                    ("b", Value::Bool(true)),
                    ("c", Value::String("hello".to_string())),
                ],
            ),
            (
                build_decode_op_with_separators(
                    DataDecodeType::PositionalLine,
                    None,
                    Some("|"),
                    None,
                ),
                Bytes::from("2026-03-03|INFO|d1|36.5"),
                vec![
                    ("col_0", Value::String("2026-03-03".to_string())),
                    ("col_1", Value::String("INFO".to_string())),
                    ("col_2", Value::String("d1".to_string())),
                    (
                        "col_3",
                        Value::Number(serde_json::Number::from_f64(36.5).unwrap()),
                    ),
                ],
            ),
            (
                build_decode_op_with_separators(
                    DataDecodeType::PositionalLine,
                    None,
                    Some(""),
                    None,
                ),
                Bytes::from("2026-03-03 INFO d1 36.5"),
                vec![
                    ("col_0", Value::String("2026-03-03".to_string())),
                    ("col_1", Value::String("INFO".to_string())),
                    ("col_2", Value::String("d1".to_string())),
                    (
                        "col_3",
                        Value::Number(serde_json::Number::from_f64(36.5).unwrap()),
                    ),
                ],
            ),
        ];

        for (op, data, expected_fields) in cases {
            let result = operator_decode_data(&op, &data).unwrap();
            let map = result.last().unwrap();
            for (field, expected_value) in expected_fields {
                assert_eq!(map.get(field), Some(&expected_value));
            }
        }
    }

    #[test]
    fn decode_csv_ok() {
        let data = Bytes::from("a,b\n1,2\n3,4");
        let op = build_decode_op(DataDecodeType::Csv);
        let result = operator_decode_data(&op, &data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].get("a").and_then(|v| v.as_i64()), Some(1));
    }
}
