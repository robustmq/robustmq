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

use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::acl::mqtt_acl::MqttAcl;

use rocksdb_engine::keys::meta::{
    storage_key_mqtt_acl, storage_key_mqtt_acl_prefix, storage_key_mqtt_acl_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_get_by_meta_metadata, engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct AclStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl AclStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        AclStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, acl: MqttAcl) -> Result<(), CommonError> {
        let resource_type_str = acl.resource_type.to_string();
        let mut acl_list = self.get(&acl.tenant, &resource_type_str, &acl.resource_name)?;

        if self.acl_exists(&acl_list, &acl) {
            return Ok(());
        }

        acl_list.push(acl.clone());

        let key = storage_key_mqtt_acl(&acl.tenant, &resource_type_str, &acl.resource_name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, acl_list)
    }

    pub fn list_all(&self) -> Result<Vec<MqttAcl>, CommonError> {
        let prefix_key = storage_key_mqtt_acl_prefix();
        let data = engine_prefix_list_by_meta_metadata::<Vec<MqttAcl>>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().flat_map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<MqttAcl>, CommonError> {
        let prefix_key = storage_key_mqtt_acl_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<Vec<MqttAcl>>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().flat_map(|raw| raw.data).collect())
    }

    pub fn delete(&self, delete_acl: &MqttAcl) -> Result<(), CommonError> {
        let resource_type_str = delete_acl.resource_type.to_string();
        let acl_list = self.get(
            &delete_acl.tenant,
            &resource_type_str,
            &delete_acl.resource_name,
        )?;

        if !self.acl_exists(&acl_list, delete_acl) {
            return Ok(());
        }

        let new_acl_list: Vec<MqttAcl> = acl_list
            .into_iter()
            .filter(|raw| !Self::acl_matches(raw, delete_acl))
            .collect();

        let key = storage_key_mqtt_acl(
            &delete_acl.tenant,
            &resource_type_str,
            &delete_acl.resource_name,
        );
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, new_acl_list)
    }

    pub fn get(
        &self,
        tenant: &str,
        resource_type: &str,
        resource_name: &str,
    ) -> Result<Vec<MqttAcl>, CommonError> {
        let key = storage_key_mqtt_acl(tenant, resource_type, resource_name);
        Ok(
            engine_get_by_meta_metadata::<Vec<MqttAcl>>(&self.rocksdb_engine_handler, &key)?
                .map(|data| data.data)
                .unwrap_or_default(),
        )
    }

    fn acl_exists(&self, acl_list: &[MqttAcl], acl: &MqttAcl) -> bool {
        acl_list.iter().any(|raw| Self::acl_matches(raw, acl))
    }

    fn acl_matches(a: &MqttAcl, b: &MqttAcl) -> bool {
        a.permission == b.permission && a.action == b.action && a.topic == b.topic && a.ip == b.ip
    }
}
