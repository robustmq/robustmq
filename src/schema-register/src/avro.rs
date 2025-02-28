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

use apache_avro::Schema;
use common_base::error::common::CommonError;

pub fn avro_validate(schema: &str, data: &[u8]) -> Result<bool, CommonError> {
    let schema = Schema::parse_str(schema)?;
    let reader = apache_avro::Reader::with_schema(&schema, data)?;
    let mut res = true;
    for record in reader {
        let raw = record?;
        let raw_res = raw.validate(&schema);
        res = res && raw_res;
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use apache_avro::{from_value, Schema, Writer};
    use serde::{Deserialize, Serialize};

    use crate::avro::avro_validate;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestData {
        a: u64,
        b: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestData2 {
        c: String,
        b: String,
    }

    #[test]
    pub fn avro_validate_test() {
        let raw_schema = r#"
                {
                    "type": "record",
                    "name": "test",
                    "fields": [
                        {"name": "a", "type": "long"},
                        {"name": "b", "type": "string"}
                    ]
                }
                "#;

        let schema = Schema::parse_str(raw_schema).unwrap();

        // build avro data
        let test_data = TestData {
            a: 1,
            b: "test".to_string(),
        };
        let mut writer = Writer::new(&schema, Vec::new());
        writer.append_ser(test_data).unwrap();
        let encoded_data = writer.into_inner().unwrap();

        let result = avro_validate(raw_schema, &encoded_data);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // build avro data
        let raw_schema1 = r#"
                {
                    "type": "record",
                    "name": "test",
                    "fields": [
                        {"name": "c", "type": "string"},
                        {"name": "b", "type": "string"}
                    ]
                }
                "#;

        let schema = Schema::parse_str(raw_schema1).unwrap();
        let test_data2 = TestData2 {
            c: "test".to_string(),
            b: "test".to_string(),
        };
        let mut writer = Writer::new(&schema, Vec::new());
        writer.append_ser(test_data2).unwrap();
        let encoded_data = writer.into_inner().unwrap();

        let result = avro_validate(raw_schema, &encoded_data);
        assert!(result.is_err());
    }

    #[test]
    pub fn avro_encode_validator_test() {
        let raw_schema = r#"
                {
                    "type": "record",
                    "name": "test",
                    "fields": [
                        {"name": "a", "type": "long"},
                        {"name": "b", "type": "string"}
                    ]
                }
                "#;
        let schema = Schema::parse_str(raw_schema).unwrap();

        // success
        let test_data = TestData {
            a: 1,
            b: "test".to_string(),
        };
        let mut writer = Writer::new(&schema, Vec::new());
        writer.append_ser(test_data).unwrap();
        let _ = writer.into_inner().unwrap();

        // error
        let test_data2 = TestData2 {
            c: "test".to_string(),
            b: "test".to_string(),
        };
        let mut writer = Writer::new(&schema, Vec::new());
        let res = writer.append_ser(test_data2);
        println!("{:?}", res);
        assert!(res.is_err());
    }

    #[test]
    pub fn avro_decode_validator_test() {
        let raw_schema = r#"
                {
                    "type": "record",
                    "name": "test",
                    "fields": [
                        {"name": "a", "type": "long"},
                        {"name": "b", "type": "string"}
                    ]
                }
                "#;
        let schema = Schema::parse_str(raw_schema).unwrap();

        // build avro data
        let test_data = TestData {
            a: 1,
            b: "test".to_string(),
        };
        let mut writer = Writer::new(&schema, Vec::new());
        writer.append_ser(test_data).unwrap(); // 序列化时校验数据是否符合模式
        let encoded_data = writer.into_inner().unwrap();

        // check avro data
        let reader = apache_avro::Reader::with_schema(&schema, &encoded_data[..]).unwrap();
        for record in reader {
            let raw = record.unwrap();
            let data: TestData = from_value(&raw).unwrap();
            println!("Deserialized data: {:?}", data);
            assert_eq!(data.a, 1);
            assert_eq!(data.b, "test".to_string());
        }
    }
}
