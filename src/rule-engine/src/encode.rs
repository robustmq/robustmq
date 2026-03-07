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
use metadata_struct::connector::rule::{DataEncodeType, ETLOperator};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

/// Output data format: serializes `Vec<Map<String, Value>>` back to `Bytes`.
///
/// This is the symmetric counterpart of `operator_decode_data`.
/// The encode type determines how the processed records are serialized for output.
pub fn operator_encode_data(
    etl_operator: &ETLOperator,
    records: Vec<Map<String, Value>>,
) -> Result<Bytes, CommonError> {
    let params = if let ETLOperator::Encode(params) = etl_operator {
        params
    } else {
        return Err(CommonError::CommonError(
            "operator_encode_data expects ETLOperator::Encode".to_string(),
        ));
    };
    match params.data_type {
        DataEncodeType::JsonObject => encode_json_object(records),
        DataEncodeType::JsonArray => encode_json_array(records),
        DataEncodeType::JsonLines => encode_json_lines(records, params.line_separator.as_deref()),
        DataEncodeType::KeyValueLine => encode_key_value_line(
            records,
            params.token_separator.as_deref(),
            params.kv_separator.as_deref(),
        ),
        DataEncodeType::PositionalLine => {
            encode_positional_line(records, params.token_separator.as_deref())
        }
        DataEncodeType::Csv => encode_csv(records),
        DataEncodeType::PlainText => encode_plain_text(records),
        DataEncodeType::Bytes => encode_raw_bytes(records),
    }
}

/// Output format: JSON Object.
/// Demo input:  `[{"device":"d1","temp":36.5}]`
/// Demo output: `{"device":"d1","temp":36.5}`
///
/// Fails if the Vec contains zero or more than one record.
fn encode_json_object(mut records: Vec<Map<String, Value>>) -> Result<Bytes, CommonError> {
    if records.len() != 1 {
        return Err(CommonError::CommonError(format!(
            "json_object encode expects exactly 1 record, got {}",
            records.len()
        )));
    }
    let map = records.remove(0);
    let bytes = serde_json::to_vec(&map)
        .map_err(|e| CommonError::CommonError(format!("failed to encode json object: {e}")))?;
    Ok(Bytes::from(bytes))
}

/// Output format: JSON Array.
/// Demo input:  `[{"device":"d1"},{"device":"d2"}]`
/// Demo output: `[{"device":"d1"},{"device":"d2"}]`
///
/// An empty Vec produces `[]`.
fn encode_json_array(records: Vec<Map<String, Value>>) -> Result<Bytes, CommonError> {
    let array: Vec<Value> = records.into_iter().map(Value::Object).collect();
    let bytes = serde_json::to_vec(&array)
        .map_err(|e| CommonError::CommonError(format!("failed to encode json array: {e}")))?;
    Ok(Bytes::from(bytes))
}

/// Output format: JSON Lines (one JSON object per line).
/// Demo input:  `[{"d":"d1"},{"d":"d2"}]`
/// Demo output: `{"d":"d1"}\n{"d":"d2"}\n`
///
/// An empty Vec produces an empty byte string.
fn encode_json_lines(
    records: Vec<Map<String, Value>>,
    line_separator: Option<&str>,
) -> Result<Bytes, CommonError> {
    let line_separator = line_separator.filter(|sep| !sep.is_empty()).unwrap_or("\n");

    let mut lines = Vec::new();
    for map in records {
        let line = serde_json::to_string(&map)
            .map_err(|e| CommonError::CommonError(format!("failed to encode json line: {e}")))?;
        lines.push(line);
    }
    Ok(Bytes::from(lines.join(line_separator).into_bytes()))
}

/// Output format: key=value line.
/// Demo input:  `[{"ts":"2026-03-03","level":"INFO","temp":36.5}]`
/// Demo output: `ts=2026-03-03 level=INFO temp=36.5`
///
/// Fails if the Vec does not contain exactly one record.
/// Empty separators fall back to defaults:
/// - token separator: space (` `)
/// - key/value separator: equals (`=`)
fn encode_key_value_line(
    mut records: Vec<Map<String, Value>>,
    token_separator: Option<&str>,
    kv_separator: Option<&str>,
) -> Result<Bytes, CommonError> {
    if records.len() != 1 {
        return Err(CommonError::CommonError(format!(
            "key_value_line encode expects exactly 1 record, got {}",
            records.len()
        )));
    }
    let token_separator = token_separator.filter(|sep| !sep.is_empty()).unwrap_or(" ");
    let kv_separator = kv_separator.filter(|sep| !sep.is_empty()).unwrap_or("=");

    let map = records.remove(0);
    let mut tokens = Vec::new();
    for (key, value) in map {
        let value_str = value_to_line_value(&value)?;
        tokens.push(format!("{key}{kv_separator}{value_str}"));
    }
    Ok(Bytes::from(tokens.join(token_separator).into_bytes()))
}

/// Output format: positional line.
/// Demo input:  `[{"col_0":"2026-03-03","col_1":"INFO","col_2":"d1","col_3":36.5}]`
/// Demo output: `2026-03-03 INFO d1 36.5`
///
/// Fails if the Vec does not contain exactly one record.
/// Empty custom separator falls back to default space (` `).
fn encode_positional_line(
    mut records: Vec<Map<String, Value>>,
    token_separator: Option<&str>,
) -> Result<Bytes, CommonError> {
    if records.len() != 1 {
        return Err(CommonError::CommonError(format!(
            "positional_line encode expects exactly 1 record, got {}",
            records.len()
        )));
    }
    let token_separator = token_separator.filter(|sep| !sep.is_empty()).unwrap_or(" ");

    let map = records.remove(0);
    let mut cols: Vec<(usize, String)> = Vec::new();
    for (key, value) in map {
        if let Some(index) = parse_col_index(&key) {
            cols.push((index, value_to_line_value(&value)?));
        }
    }
    cols.sort_by_key(|(index, _)| *index);
    let line = cols
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>()
        .join(token_separator);
    Ok(Bytes::from(line.into_bytes()))
}

/// Output format: CSV text.
/// Demo input:  `[{"ts":"2026-03-03","level":"INFO","temp":36.5}]`
/// Demo output: `ts,level,temp\n2026-03-03,INFO,36.5\n`
///
/// Header keys are collected from all records and sorted lexicographically.
/// Records missing a key emit an empty cell. An empty Vec produces an empty
/// byte string.
fn encode_csv(records: Vec<Map<String, Value>>) -> Result<Bytes, CommonError> {
    if records.is_empty() {
        return Ok(Bytes::new());
    }

    // Build a stable complete header set from all records.
    let mut header_set = BTreeSet::new();
    for map in &records {
        for key in map.keys() {
            header_set.insert(key.clone());
        }
    }
    let headers: Vec<String> = header_set.into_iter().collect();

    let mut out = String::new();
    // Header row
    out.push_str(&csv_row(headers.iter().map(|s| s.as_str())));

    // Data rows
    for map in &records {
        let cells: Vec<String> = headers
            .iter()
            .map(|h| value_to_csv_cell(map.get(h)))
            .collect::<Result<Vec<_>, _>>()?;
        out.push_str(&csv_row(cells.iter().map(|s| s.as_str())));
    }

    Ok(Bytes::from(out.into_bytes()))
}

/// Converts a JSON `Value` to its CSV cell string representation.
/// Strings are used as-is; numbers and booleans are formatted naturally;
/// null and missing fields become empty strings; objects/arrays are JSON-encoded.
fn value_to_csv_cell(value: Option<&Value>) -> Result<String, CommonError> {
    let cell = match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(v) => serde_json::to_string(v)
            .map_err(|e| CommonError::CommonError(format!("failed to encode csv cell: {e}")))?,
    };
    Ok(cell)
}

fn value_to_line_value(value: &Value) -> Result<String, CommonError> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::String(s) => Ok(s.clone()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        v => serde_json::to_string(v)
            .map_err(|e| CommonError::CommonError(format!("failed to encode line value: {e}"))),
    }
}

fn parse_col_index(key: &str) -> Option<usize> {
    key.strip_prefix("col_")?.parse::<usize>().ok()
}

/// Renders a single CSV row from an iterator of string cells, quoting cells
/// that contain commas, double-quotes, or newlines.
fn csv_row<'a>(cells: impl Iterator<Item = &'a str>) -> String {
    let mut row = String::new();
    let mut first = true;
    for cell in cells {
        if !first {
            row.push(',');
        }
        first = false;
        if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
            row.push('"');
            row.push_str(&cell.replace('"', "\"\""));
            row.push('"');
        } else {
            row.push_str(cell);
        }
    }
    row.push('\n');
    row
}

/// Output format: plain text (extracts the `message` field from the first record).
/// Demo input:  `[{"message":"device d1 temp 36.5"}]`
/// Demo output: `device d1 temp 36.5`
///
/// Fails if the Vec does not contain exactly one record or the `message` field
/// is missing in that record.
fn encode_plain_text(mut records: Vec<Map<String, Value>>) -> Result<Bytes, CommonError> {
    if records.len() != 1 {
        return Err(CommonError::CommonError(format!(
            "plain_text encode expects exactly 1 record, got {}",
            records.len()
        )));
    }
    let map = records.remove(0);
    let text = map.get("message").and_then(|v| v.as_str()).ok_or_else(|| {
        CommonError::CommonError(
            "plain_text encode expects a 'message' string field in the first record".to_string(),
        )
    })?;
    Ok(Bytes::from(text.to_string().into_bytes()))
}

/// Output format: raw bytes (extracts the `bytes` array field from the first record).
/// Demo input:  `[{"bytes":[137,80,78,71]}]`
/// Demo output: raw bytes `[137, 80, 78, 71]`
///
/// Fails if the Vec does not contain exactly one record, the `bytes` array
/// field is missing, or any array element is not a valid u8.
fn encode_raw_bytes(mut records: Vec<Map<String, Value>>) -> Result<Bytes, CommonError> {
    if records.len() != 1 {
        return Err(CommonError::CommonError(format!(
            "bytes encode expects exactly 1 record, got {}",
            records.len()
        )));
    }
    let map = records.remove(0);
    let array = map.get("bytes").and_then(|v| v.as_array()).ok_or_else(|| {
        CommonError::CommonError(
            "bytes encode expects a 'bytes' array field in the first record".to_string(),
        )
    })?;
    let mut out = Vec::with_capacity(array.len());
    for (idx, item) in array.iter().enumerate() {
        let byte = item
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .ok_or_else(|| {
                CommonError::CommonError(format!(
                    "bytes encode: element at index {idx} is not a valid u8"
                ))
            })?;
        out.push(byte);
    }
    Ok(Bytes::from(out))
}

#[cfg(test)]
mod tests {
    use super::{operator_encode_data, DataEncodeType};
    use metadata_struct::connector::rule::{ETLOperator, EncodeDeleteParams};
    use serde_json::{json, Map, Value};

    fn build_encode_op(data_type: DataEncodeType) -> ETLOperator {
        ETLOperator::Encode(EncodeDeleteParams {
            data_type,
            line_separator: None,
            token_separator: None,
            kv_separator: None,
        })
    }

    fn build_encode_op_with_separators(
        data_type: DataEncodeType,
        line_separator: Option<&str>,
        token_separator: Option<&str>,
        kv_separator: Option<&str>,
    ) -> ETLOperator {
        ETLOperator::Encode(EncodeDeleteParams {
            data_type,
            line_separator: line_separator.map(|v| v.to_string()),
            token_separator: token_separator.map(|v| v.to_string()),
            kv_separator: kv_separator.map(|v| v.to_string()),
        })
    }

    fn make_record(pairs: &[(&str, Value)]) -> Map<String, Value> {
        let mut map = Map::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), v.clone());
        }
        map
    }

    #[test]
    fn encode_json_object_behaviors() {
        let op = build_encode_op(DataEncodeType::JsonObject);
        let ok_records = vec![make_record(&[("k", json!("v")), ("n", json!(1))])];
        let ok_result = operator_encode_data(&op, ok_records).unwrap();
        let parsed: Value = serde_json::from_slice(&ok_result).unwrap();
        assert_eq!(parsed["k"], json!("v"));
        assert_eq!(parsed["n"], json!(1));

        let err_records = vec![
            make_record(&[("k", json!("a"))]),
            make_record(&[("k", json!("b"))]),
        ];
        let err = operator_encode_data(&op, err_records).unwrap_err();
        assert!(err.to_string().contains("exactly 1 record"));
    }

    #[test]
    fn encode_json_array_and_json_lines_behaviors() {
        let array_records = vec![
            make_record(&[("k", json!("a"))]),
            make_record(&[("k", json!("b"))]),
        ];
        let array_op = build_encode_op(DataEncodeType::JsonArray);
        let array_result = operator_encode_data(&array_op, array_records).unwrap();
        let parsed: Value = serde_json::from_slice(&array_result).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);

        let empty_array_result = operator_encode_data(&array_op, vec![]).unwrap();
        assert_eq!(&empty_array_result[..], b"[]");

        let lines_records = vec![
            make_record(&[("d", json!("d1"))]),
            make_record(&[("d", json!("d2"))]),
        ];
        let lines_op = build_encode_op(DataEncodeType::JsonLines);
        let lines_result = operator_encode_data(&lines_op, lines_records.clone()).unwrap();
        let default_lines: Vec<&str> = std::str::from_utf8(&lines_result)
            .unwrap()
            .split('\n')
            .collect();
        assert_eq!(default_lines.len(), 2);
        let first: Value = serde_json::from_str(default_lines[0]).unwrap();
        assert_eq!(first["d"], json!("d1"));

        let custom_op =
            build_encode_op_with_separators(DataEncodeType::JsonLines, Some("||"), None, None);
        let custom_result = operator_encode_data(&custom_op, lines_records).unwrap();
        let custom_lines: Vec<&str> = std::str::from_utf8(&custom_result)
            .unwrap()
            .split("||")
            .collect();
        assert_eq!(custom_lines.len(), 2);
        let second: Value = serde_json::from_str(custom_lines[1]).unwrap();
        assert_eq!(second["d"], json!("d2"));
    }

    #[test]
    fn encode_csv_behaviors() {
        let basic_records = vec![
            make_record(&[("ts", json!("2026-03-03")), ("temp", json!(36.5))]),
            make_record(&[("ts", json!("2026-03-04")), ("temp", json!(37.0))]),
        ];
        let op = build_encode_op(DataEncodeType::Csv);
        let basic_result = operator_encode_data(&op, basic_records).unwrap();
        let basic_lines: Vec<&str> = std::str::from_utf8(&basic_result)
            .unwrap()
            .lines()
            .collect();
        assert_eq!(basic_lines[0], "temp,ts");
        assert_eq!(basic_lines[1], "36.5,2026-03-03");

        let all_keys_records = vec![
            make_record(&[("a", json!(1)), ("b", json!(2))]),
            make_record(&[("a", json!(3)), ("c", json!(4))]),
        ];
        let all_keys_result = operator_encode_data(&op, all_keys_records).unwrap();
        let all_key_lines: Vec<&str> = std::str::from_utf8(&all_keys_result)
            .unwrap()
            .lines()
            .collect();
        assert_eq!(all_key_lines[0], "a,b,c");
        assert_eq!(all_key_lines[1], "1,2,");
        assert_eq!(all_key_lines[2], "3,,4");
    }

    #[test]
    fn encode_line_format_separator_cases() {
        let kv_records = vec![make_record(&[
            ("a", json!(1)),
            ("b", json!(true)),
            ("c", json!("hello")),
        ])];
        let kv_custom = build_encode_op_with_separators(
            DataEncodeType::KeyValueLine,
            None,
            Some(";"),
            Some(":"),
        );
        let kv_custom_text =
            std::str::from_utf8(&operator_encode_data(&kv_custom, kv_records.clone()).unwrap())
                .unwrap()
                .to_string();
        assert!(kv_custom_text.contains("a:1"));
        assert!(kv_custom_text.contains("b:true"));
        assert!(kv_custom_text.contains("c:hello"));
        assert!(kv_custom_text.contains(';'));

        let kv_fallback =
            build_encode_op_with_separators(DataEncodeType::KeyValueLine, None, Some(""), Some(""));
        let kv_fallback_text =
            std::str::from_utf8(&operator_encode_data(&kv_fallback, kv_records).unwrap())
                .unwrap()
                .to_string();
        assert!(kv_fallback_text.contains("a=1"));
        assert!(kv_fallback_text.contains("b=true"));
        assert!(kv_fallback_text.contains("c=hello"));
        assert!(kv_fallback_text.contains(' '));

        let positional_records = vec![make_record(&[
            ("col_0", json!("2026-03-03")),
            ("col_1", json!("INFO")),
            ("col_2", json!("d1")),
            ("col_3", json!(36.5)),
        ])];
        let positional_custom =
            build_encode_op_with_separators(DataEncodeType::PositionalLine, None, Some("|"), None);
        let positional_custom_text = std::str::from_utf8(
            &operator_encode_data(&positional_custom, positional_records.clone()).unwrap(),
        )
        .unwrap()
        .to_string();
        assert_eq!(positional_custom_text, "2026-03-03|INFO|d1|36.5");

        let positional_fallback =
            build_encode_op_with_separators(DataEncodeType::PositionalLine, None, Some(""), None);
        let positional_fallback_text = std::str::from_utf8(
            &operator_encode_data(&positional_fallback, positional_records).unwrap(),
        )
        .unwrap()
        .to_string();
        assert_eq!(positional_fallback_text, "2026-03-03 INFO d1 36.5");
    }

    #[test]
    fn encode_single_record_only_behaviors() {
        let plain_op = build_encode_op(DataEncodeType::PlainText);
        let plain_ok = vec![make_record(&[("message", json!("hello world"))])];
        let plain_ok_result = operator_encode_data(&plain_op, plain_ok).unwrap();
        assert_eq!(&plain_ok_result[..], b"hello world");

        let plain_err = vec![
            make_record(&[("message", json!("hello"))]),
            make_record(&[("message", json!("world"))]),
        ];
        let plain_err_result = operator_encode_data(&plain_op, plain_err).unwrap_err();
        assert!(plain_err_result.to_string().contains("exactly 1 record"));

        let bytes_op = build_encode_op(DataEncodeType::Bytes);
        let bytes_ok = vec![make_record(&[("bytes", json!([137, 80, 78, 71]))])];
        let bytes_ok_result = operator_encode_data(&bytes_op, bytes_ok).unwrap();
        assert_eq!(&bytes_ok_result[..], &[137u8, 80, 78, 71]);

        let bytes_err = vec![
            make_record(&[("bytes", json!([1, 2]))]),
            make_record(&[("bytes", json!([3, 4]))]),
        ];
        let bytes_err_result = operator_encode_data(&bytes_op, bytes_err).unwrap_err();
        assert!(bytes_err_result.to_string().contains("exactly 1 record"));
    }
}
