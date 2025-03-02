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
use dashmap::DashMap;
use metadata_struct::schema::{SchemaData, SchemaType};

use crate::{avro::avro_validate, json::json_validate};

#[derive(Default)]
pub struct SchemaRegisterManager {
    // (SchemaName, SchemaData)
    schema_list: DashMap<String, SchemaData>,
    // (Resource, Vec<SchemaName>)
    schema_resource_list: DashMap<String, Vec<String>>,
}

impl SchemaRegisterManager {
    pub fn new() -> Self {
        SchemaRegisterManager {
            schema_list: DashMap::with_capacity(2),
            schema_resource_list: DashMap::with_capacity(2),
        }
    }

    pub fn is_check_schema(&self, topic: &str) -> bool {
        if let Some(list) = self.schema_resource_list.get(topic) {
            return list.is_empty();
        }
        false
    }

    pub fn validate(&self, resource: &str, data: &[u8]) -> Result<bool, CommonError> {
        if let Some(schemc_list) = self.schema_resource_list.get(resource) {
            for schema_name in schemc_list.iter() {
                if let Some(schema) = self.schema_list.get(schema_name) {
                    match schema.schema_type {
                        SchemaType::JSON => {
                            let raw = serde_json::from_slice::<String>(data)?;
                            return json_validate(&schema.schema, &raw);
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

    // Schema Resource
    pub fn add_schema_resource(&self, topic: &str, schema_name: &str) {
        if let Some(mut list) = self.schema_resource_list.get_mut(schema_name) {
            if !list.contains(&schema_name.to_owned()) {
                list.push(schema_name.to_owned());
            }
        } else {
            self.schema_resource_list
                .insert(topic.to_owned(), vec![schema_name.to_owned()]);
        }
    }

    pub fn remove_resource(&self, resource: &str) {
        self.schema_resource_list.remove(resource);
    }

    pub fn remove_resource_schema(&self, resource: &str, schema_name: &str) {
        if let Some(mut list) = self.schema_resource_list.get_mut(resource) {
            list.retain(|x| x != schema_name);
        }
    }
}
