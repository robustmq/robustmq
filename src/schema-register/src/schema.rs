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
use std::sync::Arc;

use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::schema::{SchemaData, SchemaResourceBind, SchemaType};

use crate::{avro::avro_validate, json::json_validate};

struct TenantSchemas {
    // schema_name -> SchemaData
    schema_list: DashMap<String, SchemaData>,
    // resource_name -> [schema_name]
    resource_schema_list: DashMap<String, Vec<String>>,
    // schema_name -> [resource_name]
    schema_resource_list: DashMap<String, Vec<String>>,
}

impl TenantSchemas {
    fn new() -> Self {
        TenantSchemas {
            schema_list: DashMap::new(),
            resource_schema_list: DashMap::new(),
            schema_resource_list: DashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct SchemaRegisterManager {
    // tenant -> TenantSchemas
    tenants: DashMap<String, Arc<TenantSchemas>>,
}

impl SchemaRegisterManager {
    pub fn new() -> Self {
        SchemaRegisterManager {
            tenants: DashMap::new(),
        }
    }

    fn get_or_create_tenant(&self, tenant: &str) -> Arc<TenantSchemas> {
        if let Some(t) = self.tenants.get(tenant) {
            return t.clone();
        }
        let t = Arc::new(TenantSchemas::new());
        self.tenants.insert(tenant.to_string(), t.clone());
        t
    }

    fn get_tenant(&self, tenant: &str) -> Option<Arc<TenantSchemas>> {
        self.tenants.get(tenant).map(|t| t.clone())
    }

    pub fn is_check_schema(&self, tenant: &str, resource: &str) -> bool {
        if let Some(t) = self.get_tenant(tenant) {
            if let Some(list) = t.resource_schema_list.get(resource) {
                return !list.is_empty();
            }
        }
        false
    }

    pub fn validate(&self, tenant: &str, resource: &str, data: &[u8]) -> Result<bool, CommonError> {
        if let Some(t) = self.get_tenant(tenant) {
            if let Some(schema_list) = t.resource_schema_list.get(resource) {
                for schema_name in schema_list.iter() {
                    if let Some(schema) = t.schema_list.get(schema_name.as_str()) {
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
        }
        Ok(true)
    }

    // Schema
    pub fn add_schema(&self, schema: SchemaData) {
        let t = self.get_or_create_tenant(&schema.tenant.clone());
        t.schema_list.insert(schema.name.clone(), schema);
    }

    pub fn remove_schema(&self, tenant: &str, schema_name: &str) {
        if let Some(t) = self.get_tenant(tenant) {
            t.schema_list.remove(schema_name);
        }
    }

    pub fn get_schema(&self, tenant: &str, schema_name: &str) -> Option<SchemaData> {
        self.get_tenant(tenant)
            .and_then(|t| t.schema_list.get(schema_name).map(|s| s.clone()))
    }

    pub fn get_all_schema(&self, tenant: &str) -> Vec<SchemaData> {
        if let Some(t) = self.get_tenant(tenant) {
            t.schema_list.iter().map(|e| e.value().clone()).collect()
        } else {
            vec![]
        }
    }

    // Schema Resource Bind
    pub fn add_bind(&self, bind: &SchemaResourceBind) {
        let t = self.get_or_create_tenant(&bind.tenant);
        let schema_name = bind.schema_name.clone();
        let resource_name = bind.resource_name.clone();

        t.resource_schema_list
            .entry(resource_name.clone())
            .and_modify(|list| {
                if !list.contains(&schema_name) {
                    list.push(schema_name.clone());
                }
            })
            .or_insert_with(|| vec![schema_name.clone()]);

        t.schema_resource_list
            .entry(schema_name)
            .and_modify(|list| {
                if !list.contains(&resource_name) {
                    list.push(resource_name.clone());
                }
            })
            .or_insert_with(|| vec![resource_name]);
    }

    pub fn remove_bind(&self, bind: &SchemaResourceBind) {
        if let Some(t) = self.get_tenant(&bind.tenant) {
            t.resource_schema_list.remove(&bind.resource_name);
            t.schema_resource_list.remove(&bind.schema_name);
        }
    }

    pub fn get_bind_schema_by_resource(
        &self,
        tenant: &str,
        resource_name: &str,
    ) -> Vec<SchemaData> {
        if let Some(t) = self.get_tenant(tenant) {
            if let Some(list) = t.resource_schema_list.get(resource_name) {
                let mut res = Vec::new();
                for schema_name in list.iter() {
                    if let Some(schema) = t.schema_list.get(schema_name.as_str()) {
                        res.push(schema.clone());
                    }
                }
                return res;
            }
        }
        vec![]
    }

    pub fn get_bind_resource_by_schema(&self, tenant: &str, schema_name: &str) -> Vec<String> {
        if let Some(t) = self.get_tenant(tenant) {
            if let Some(list) = t.schema_resource_list.get(schema_name) {
                return list.clone();
            }
        }
        vec![]
    }
}
