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
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

/// Source data format: delegated by `DataDecodeType`.
/// Demo input: `{"device":"d1","temp":36.5}`
/// Demo output: `[{"device":"d1","temp":36.5}]`
pub fn operator_decode_data(
    data: Bytes,
    decode_type: &DataDecodeType,
) -> Result<Vec<Map<String, Value>>, CommonError> {
    match decode_type {
        DataDecodeType::JsonObject => decode_json_object(&data),
        DataDecodeType::JsonArray => decode_json_array(&data),
        DataDecodeType::JsonLines => decode_json_lines(&data),
        DataDecodeType::KeyValueLine => decode_key_value_line(&data),
        DataDecodeType::PositionalLine => decode_positional_line(&data),
        DataDecodeType::Csv => decode_csv(&data),
        DataDecodeType::PlainText => decode_plain_text(&data),
        DataDecodeType::Bytes => decode_raw_bytes(&data),
        DataDecodeType::Protobuf => Err(CommonError::CommonError(
            "protobuf decode is not supported yet".to_string(),
        )),
        DataDecodeType::Xml => Err(CommonError::CommonError(
            "xml decode is not supported yet".to_string(),
        )),
    }
}

/// Source data format: JSON Object.
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

/// Source data format: JSON Array (array of objects).
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

/// Source data format: JSON Lines (one JSON object per line).
/// Demo input: `{"d":"d1"}\n{"d":"d2"}`
/// Demo output: `[{"d":"d1"},{"d":"d2"}]`
fn decode_json_lines(data: &[u8]) -> Result<Vec<Map<String, Value>>, CommonError> {
    let text = std::str::from_utf8(data).map_err(|e| {
        CommonError::CommonError(format!("failed to parse json lines as utf-8 text: {e}"))
    })?;

    let mut records = Vec::new();
    for (idx, line) in text.lines().enumerate() {
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

/// Source data format: key=value line.
/// Demo input: `ts=2026-03-03 level=INFO temp=36.5`
/// Demo output: `[{"ts":"2026-03-03","level":"INFO","temp":36.5}]`
fn decode_key_value_line(data: &[u8]) -> Result<Vec<Map<String, Value>>, CommonError> {
    let text = std::str::from_utf8(data).map_err(|e| {
        CommonError::CommonError(format!("failed to parse key=value line as utf-8 text: {e}"))
    })?;

    let mut map = Map::new();
    for token in text.split_whitespace() {
        if token.is_empty() {
            continue;
        }
        let (key, raw_value) = token
            .split_once('=')
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

/// Source data format: positional line (space-delimited, no keys).
/// Demo input: `2026-03-03 INFO d1 36.5`
/// Demo output: `[{"col_0":"2026-03-03","col_1":"INFO","col_2":"d1","col_3":36.5}]`
fn decode_positional_line(data: &[u8]) -> Result<Vec<Map<String, Value>>, CommonError> {
    let text = std::str::from_utf8(data).map_err(|e| {
        CommonError::CommonError(format!(
            "failed to parse positional line as utf-8 text: {e}"
        ))
    })?;

    let mut map = Map::new();
    for (idx, token) in text.split_whitespace().enumerate() {
        map.insert(format!("col_{idx}"), parse_scalar_value(token));
    }
    if map.is_empty() {
        return Err(CommonError::CommonError(
            "positional_line decode got empty input".to_string(),
        ));
    }
    Ok(vec![map])
}

/// Source data format: CSV text (first row is header).
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

/// Source data format: plain text.
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

/// Source data format: raw bytes.
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

/// Scalar parser used by line/csv formats (bool/int/float/string).
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

    #[test]
    fn decode_json_object_ok() {
        let data = Bytes::from(r#"{"k":"v","n":1}"#);
        let result = operator_decode_data(data, &DataDecodeType::JsonObject).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("k").and_then(|v| v.as_str()), Some("v"));
        assert_eq!(result[0].get("n").and_then(|v| v.as_i64()), Some(1));
    }

    #[test]
    fn decode_json_array_ok() {
        let data = Bytes::from(r#"[{"k":"a"},{"k":"b"}]"#);
        let result = operator_decode_data(data, &DataDecodeType::JsonArray).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn decode_key_value_line_ok() {
        let data = Bytes::from("a=1 b=true c=hello");
        let result = operator_decode_data(data, &DataDecodeType::KeyValueLine).unwrap();
        assert_eq!(result.len(), 1);
        let map = &result[0];
        assert_eq!(map.get("a").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(map.get("b").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(map.get("c").and_then(|v| v.as_str()), Some("hello"));
    }

    #[test]
    fn decode_csv_ok() {
        let data = Bytes::from("a,b\n1,2\n3,4");
        let result = operator_decode_data(data, &DataDecodeType::Csv).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].get("a").and_then(|v| v.as_i64()), Some(1));
    }

    #[test]
    fn decode_positional_line_empty_err() {
        let data = Bytes::from("   ");
        let result = operator_decode_data(data, &DataDecodeType::PositionalLine);
        assert!(result.is_err());
    }

    #[test]
    fn decode_json_array_non_object_element_has_index() {
        let data = Bytes::from(r#"[{"k":"v"}, 42]"#);
        let err = operator_decode_data(data, &DataDecodeType::JsonArray).unwrap_err();
        assert!(err.to_string().contains("index: 1"));
    }
}
