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

use crate::storage::engine::{
    engine_get_by_cluster, engine_prefix_list_by_cluster, engine_save_by_cluster,
};
use crate::storage::keys::{storage_key_mqtt_acl, storage_key_mqtt_acl_prefix};
use crate::storage::rocksdb::RocksDBEngine;

pub struct AclStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl AclStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        AclStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, cluster_name: &str, acl: MqttAcl) -> Result<(), CommonError> {
        let mut acl_list = self.get(
            cluster_name,
            &acl.resource_type.to_string(),
            &acl.resource_name,
        )?;

        if self.acl_exists(&acl_list, &acl) {
            return Ok(());
        }

        acl_list.push(acl.clone());

        let key = storage_key_mqtt_acl(
            cluster_name,
            &acl.resource_type.to_string(),
            &acl.resource_name,
        );
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, acl_list)
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<MqttAcl>, CommonError> {
        let prefix_key = storage_key_mqtt_acl_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let acl_list = serde_json::from_slice::<Vec<MqttAcl>>(&raw.data)?;
            results.extend(acl_list);
        }
        Ok(results)
    }

    pub fn delete(&self, cluster_name: &str, delete_acl: &MqttAcl) -> Result<(), CommonError> {
        let acl_list = self.get(
            cluster_name,
            &delete_acl.resource_type.to_string(),
            &delete_acl.resource_name,
        )?;

        if !self.acl_exists(&acl_list, delete_acl) {
            return Ok(());
        }

        let mut new_acl_list = Vec::new();
        for raw in acl_list {
            if !(raw.permission == delete_acl.permission
                && raw.action == delete_acl.action
                && raw.topic == delete_acl.topic
                && raw.ip == delete_acl.ip)
            {
                new_acl_list.push(raw);
            }
        }
        let key = storage_key_mqtt_acl(
            cluster_name,
            &delete_acl.resource_type.to_string(),
            &delete_acl.resource_name,
        );
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, new_acl_list)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        resource_type: &str,
        resource_name: &str,
    ) -> Result<Vec<MqttAcl>, CommonError> {
        let key = storage_key_mqtt_acl(cluster_name, resource_type, resource_name);
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            return Ok(serde_json::from_slice::<Vec<MqttAcl>>(&data.data)?);
        }
        Ok(Vec::new())
    }

    fn acl_exists(&self, acl_list: &[MqttAcl], acl: &MqttAcl) -> bool {
        for raw in acl_list {
            if !(raw.permission == acl.permission
                && raw.action == acl.action
                && raw.topic == acl.topic
                && raw.ip == acl.ip)
            {
                return true;
            }
        }
        false
    }
}
