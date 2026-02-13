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

use std::str::from_utf8;

use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::schema::{SchemaData, SchemaResourceBind, SchemaType};

use crate::{avro::avro_validate, json::json_validate};

#[derive(Default)]
pub struct SchemaRegisterManager {
    // (SchemaName, SchemaData)
    pub schema_list: DashMap<String, SchemaData>,
    // (Resource, Vec<SchemaName>)
    pub resource_schema_list: DashMap<String, Vec<String>>,
    // (Schema, Vec<Resource>)
    pub schema_resource_list: DashMap<String, Vec<String>>,
}

impl SchemaRegisterManager {
    pub fn new() -> Self {
        SchemaRegisterManager {
            schema_list: DashMap::with_capacity(2),
            resource_schema_list: DashMap::with_capacity(2),
            schema_resource_list: DashMap::with_capacity(2),
        }
    }

    pub fn is_check_schema(&self, topic: &str) -> bool {
        if let Some(list) = self.resource_schema_list.get(topic) {
            return !list.is_empty();
        }
        false
    }

    pub fn validate(&self, resource: &str, data: &[u8]) -> Result<bool, CommonError> {
        if let Some(schema_list) = self.resource_schema_list.get(resource) {
            for schema_name in schema_list.iter() {
                if let Some(schema) = self.schema_list.get(schema_name) {
                    match schema.schema_type {
                        SchemaType::JSON => {
                            let raw = from_utf8(data).unwrap();
                            return json_validate(&schema.schema, raw);
                        }
                        SchemaType::PROTOBUF => {}
                        SchemaType::AVRO => {
                            return avro_validate(&schema.schema, data);
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    // Schema
    pub fn add_schema(&self, schema: SchemaData) {
        self.schema_list.insert(schema.name.clone(), schema);
    }

    pub fn remove_schema(&self, schema_name: &str) {
        self.schema_list.remove(schema_name);
    }

    pub fn get_schema(&self, schema_name: &str) -> Option<SchemaData> {
        if let Some(schema) = self.schema_list.get(schema_name) {
            return Some(schema.clone());
        }
        None
    }

    pub fn get_all_schema(&self) -> Vec<SchemaData> {
        let mut list = Vec::new();
        for schema in self.schema_list.iter() {
            list.push(schema.clone());
        }
        list
    }

    // Schema Resource
    pub fn add_bind(&self, schema_resource: &SchemaResourceBind) {
        let schema_name = schema_resource.schema_name.clone();
        let resource_name = schema_resource.resource_name.clone();

        // resource_schema_list
        if let Some(mut list) = self.resource_schema_list.get_mut(&resource_name) {
            if !list.contains(&schema_name) {
                list.push(schema_name.to_owned());
            }
        } else {
            self.resource_schema_list
                .insert(resource_name.clone(), vec![schema_name.to_owned()]);
        }

        // schema_resource_list
        if let Some(mut list) = self.schema_resource_list.get_mut(&schema_name) {
            if !list.contains(&resource_name) {
                list.push(resource_name);
            }
        } else {
            self.schema_resource_list
                .insert(schema_name, vec![resource_name]);
        }
    }

    pub fn remove_bind(&self, bind: &SchemaResourceBind) {
        self.resource_schema_list.remove(&bind.resource_name);
        self.schema_resource_list.remove(&bind.schema_name);
    }

    pub fn get_bind_schema_by_resource(&self, resource_name: &str) -> Vec<SchemaData> {
        if let Some(list) = self.resource_schema_list.get(resource_name) {
            let mut res = Vec::new();
            for schema_name in list.iter() {
                if let Some(schema) = self.schema_list.get(schema_name) {
                    res.push(schema.clone());
                }
            }
            res
        } else {
            vec![]
        }
    }

    pub fn get_bind_resource_by_schema(&self, schema_name: &str) -> Vec<String> {
        if let Some(list) = self.schema_resource_list.get(schema_name) {
            list.clone()
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod test {
    use super::SchemaRegisterManager;
    use apache_avro::{Schema, Writer};
    use metadata_struct::schema::{SchemaData, SchemaResourceBind, SchemaType};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[test]
    pub fn json_schema_test() {
        let schema_manager = SchemaRegisterManager::new();
        let schema_name = "schema1".to_string();
        let schema_json_content = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name"]
        }"#;
        schema_manager.add_schema(SchemaData {
            name: schema_name.clone(),
            schema: schema_json_content.to_string(),
            schema_type: SchemaType::JSON,
            desc: "test".to_string(),
        });

        let topic_name = "t1".to_string();
        let bind_schema = SchemaResourceBind {
            resource_name: topic_name.clone(),
            schema_name: schema_name.clone(),
        };
        schema_manager.add_bind(&bind_schema);

        assert!(schema_manager.is_check_schema(&topic_name));

        let topic_name1 = "t2".to_string();
        assert!(!schema_manager.is_check_schema(&topic_name1));

        let message_content = json!({"name": "John Doe","age": 30}).to_string();

        let result = schema_manager.validate(&topic_name, message_content.as_bytes());
        println!("{result:?}");
        assert!(result.is_ok());
        assert!(result.unwrap());

        let message_content = json!({"age": 30}).to_string();

        let result = schema_manager.validate(&topic_name, message_content.as_bytes());
        println!("{result:?}");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let message_content = json!({"name": "John Doe"}).to_string();

        let result = schema_manager.validate(&topic_name, message_content.as_bytes());
        println!("{result:?}");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

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
    pub fn avro_schema_test() {
        let schema_manager = SchemaRegisterManager::new();
        let schema_name = "schema1".to_string();
        let schema_avro_content = r#"
        {
            "type": "record",
            "name": "test",
            "fields": [
                {"name": "a", "type": "long"},
                {"name": "b", "type": "string"}
            ]
        }
        "#;

        schema_manager.add_schema(SchemaData {
            name: schema_name.clone(),
            schema: schema_avro_content.to_string(),
            schema_type: SchemaType::AVRO,
            desc: "test".to_string(),
        });

        let topic_name = "t1".to_string();
        let bind_schema = SchemaResourceBind {
            resource_name: topic_name.clone(),
            schema_name: schema_name.clone(),
        };
        schema_manager.add_bind(&bind_schema);

        assert!(schema_manager.is_check_schema(&topic_name));

        let topic_name1 = "t2".to_string();
        assert!(!schema_manager.is_check_schema(&topic_name1));

        // build avro data
        let test_data = TestData {
            a: 1,
            b: "test".to_string(),
        };

        let schema = Schema::parse_str(schema_avro_content).unwrap();
        let mut writer = Writer::new(&schema, Vec::new());
        writer.append_ser(test_data).unwrap(); // 序列化时校验数据是否符合模式
        let encoded_data = writer.into_inner().unwrap();

        let result = schema_manager.validate(&topic_name, encoded_data.as_slice());
        println!("{result:?}");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // build avro data
        let test_data = TestData2 {
            b: "test".to_string(),
            c: "test".to_string(),
        };
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
        let mut writer = Writer::new(&schema, Vec::new());
        writer.append_ser(test_data).unwrap(); // 序列化时校验数据是否符合模式
        let encoded_data = writer.into_inner().unwrap();

        let result = schema_manager.validate(&topic_name, encoded_data.as_slice());
        println!("{result:?}");
        assert!(result.is_err());
    }
}
