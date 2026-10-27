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

    pub fn get_all_schema_all_tenants(&self) -> Vec<(String, SchemaData)> {
        self.tenants
            .iter()
            .flat_map(|entry| {
                let tenant = entry.key().clone();
                entry
                    .value()
                    .schema_list
                    .iter()
                    .map(move |s| (tenant.clone(), s.value().clone()))
                    .collect::<Vec<_>>()
            })
            .collect()
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

#[cfg(test)]
mod test {
    use apache_avro::{Schema, Writer};
    use serde::Serialize;

    use super::SchemaRegisterManager;
    use metadata_struct::schema::{SchemaData, SchemaResourceBind, SchemaType};

    fn schema_data(tenant: &str, name: &str, schema_type: SchemaType, schema: &str) -> SchemaData {
        SchemaData {
            tenant: tenant.to_string(),
            name: name.to_string(),
            schema_type,
            desc: format!("{name} schema"),
            schema: schema.to_string(),
        }
    }

    fn bind(tenant: &str, schema_name: &str, resource_name: &str) -> SchemaResourceBind {
        SchemaResourceBind {
            tenant: tenant.to_string(),
            schema_name: schema_name.to_string(),
            resource_name: resource_name.to_string(),
        }
    }

    #[derive(Serialize)]
    struct AvroRecord {
        id: i64,
        name: String,
    }

    #[test]
    fn schema_lifecycle_is_scoped_by_tenant() {
        let manager = SchemaRegisterManager::new();
        let schema = schema_data(
            "tenant-a",
            "user",
            SchemaType::JSON,
            r#"{"type":"object","properties":{"id":{"type":"integer"}}}"#,
        );

        assert!(manager.get_schema("tenant-a", "user").is_none());
        assert!(manager.get_all_schema("tenant-a").is_empty());
        assert!(manager.get_all_schema_all_tenants().is_empty());

        manager.add_schema(schema.clone());

        assert_eq!(manager.get_schema("tenant-a", "user"), Some(schema.clone()));
        assert_eq!(manager.get_all_schema("tenant-a"), vec![schema.clone()]);
        assert_eq!(manager.get_all_schema("tenant-b"), Vec::new());
        assert_eq!(
            manager.get_all_schema_all_tenants(),
            vec![("tenant-a".to_string(), schema)]
        );

        manager.remove_schema("tenant-b", "user");
        assert!(manager.get_schema("tenant-a", "user").is_some());
        manager.remove_schema("tenant-a", "user");
        assert!(manager.get_schema("tenant-a", "user").is_none());
    }

    #[test]
    fn bindings_are_duplicate_safe_and_support_reverse_lookups() {
        let manager = SchemaRegisterManager::new();
        manager.add_schema(schema_data(
            "tenant-a",
            "user",
            SchemaType::JSON,
            r#"{"type":"object"}"#,
        ));
        manager.add_schema(schema_data(
            "tenant-a",
            "event",
            SchemaType::JSON,
            r#"{"type":"object"}"#,
        ));

        let user_binding = bind("tenant-a", "user", "users");
        let event_binding = bind("tenant-a", "event", "events");
        manager.add_bind(&user_binding);
        manager.add_bind(&user_binding);
        manager.add_bind(&event_binding);

        assert!(manager.is_check_schema("tenant-a", "users"));
        assert!(!manager.is_check_schema("tenant-a", "missing"));
        assert_eq!(
            manager.get_bind_resource_by_schema("tenant-a", "user"),
            vec!["users"]
        );
        assert_eq!(
            manager.get_bind_resource_by_schema("tenant-a", "event"),
            vec!["events"]
        );

        assert_eq!(
            manager
                .get_bind_schema_by_resource("tenant-a", "users")
                .iter()
                .map(|schema| schema.name.as_str())
                .collect::<Vec<_>>(),
            vec!["user"]
        );
        assert_eq!(
            manager
                .get_bind_schema_by_resource("tenant-a", "events")
                .iter()
                .map(|schema| schema.name.as_str())
                .collect::<Vec<_>>(),
            vec!["event"]
        );

        manager.remove_bind(&user_binding);
        assert!(!manager.is_check_schema("tenant-a", "users"));
        assert!(manager
            .get_bind_schema_by_resource("tenant-a", "users")
            .is_empty());
        assert!(manager
            .get_bind_resource_by_schema("tenant-a", "user")
            .is_empty());
        assert!(manager.is_check_schema("tenant-a", "events"));
    }

    #[test]
    fn validate_returns_true_without_a_binding() {
        let manager = SchemaRegisterManager::new();
        assert!(manager.validate("tenant-a", "users", b"not-json").unwrap());

        manager.add_schema(schema_data(
            "tenant-a",
            "user",
            SchemaType::JSON,
            r#"{"type":"object"}"#,
        ));
        assert!(manager.validate("tenant-a", "users", b"not-json").unwrap());
    }

    #[test]
    fn validate_uses_bound_json_schema() {
        let manager = SchemaRegisterManager::new();
        manager.add_schema(schema_data(
            "tenant-a",
            "user",
            SchemaType::JSON,
            r#"{"type":"object","required":["name"],"properties":{"name":{"type":"string"}}}"#,
        ));
        manager.add_bind(&bind("tenant-a", "user", "users"));

        assert!(manager
            .validate("tenant-a", "users", br#"{"name":"Ada"}"#)
            .unwrap());
        assert!(!manager
            .validate("tenant-a", "users", br#"{"name":42}"#)
            .unwrap());
        assert!(manager.validate("tenant-a", "users", b"not-json").is_err());
    }

    #[test]
    fn validate_uses_bound_avro_schema() {
        let manager = SchemaRegisterManager::new();
        let raw_schema = r#"
            {"type":"record","name":"user","fields":[
                {"name":"id","type":"long"},
                {"name":"name","type":"string"}
            ]}
        "#;
        manager.add_schema(schema_data(
            "tenant-a",
            "user",
            SchemaType::AVRO,
            raw_schema,
        ));
        manager.add_bind(&bind("tenant-a", "user", "users"));

        let schema = Schema::parse_str(raw_schema).unwrap();
        let mut writer = Writer::new(&schema, Vec::new());
        writer
            .append_ser(AvroRecord {
                id: 1,
                name: "Ada".to_string(),
            })
            .unwrap();
        let encoded = writer.into_inner().unwrap();

        assert!(manager.validate("tenant-a", "users", &encoded).unwrap());
        assert!(manager.validate("tenant-a", "users", b"invalid").is_err());
    }

    #[test]
    fn protobuf_bindings_follow_the_current_manager_behavior() {
        let manager = SchemaRegisterManager::new();
        manager.add_schema(schema_data(
            "tenant-a",
            "user",
            SchemaType::PROTOBUF,
            "syntax = \"proto3\"; message User { string name = 1; }",
        ));
        manager.add_bind(&bind("tenant-a", "user", "users"));

        // Protobuf decoding is exposed through protobuf_validate; manager validation
        // currently treats a protobuf binding as accepted data.
        assert!(manager.validate("tenant-a", "users", b"arbitrary").unwrap());
    }
}
