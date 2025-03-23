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
            let acl_list = serde_json::from_str::<Vec<MqttAcl>>(&raw.data.to_string())?;
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
            return Ok(serde_json::from_str::<Vec<MqttAcl>>(&data.data)?);
        }
        Ok(Vec::new())
    }

    fn acl_exists(&self, acl_list: &[MqttAcl], acl: &MqttAcl) -> bool {
        for raw in acl_list {
            if raw.permission == acl.permission
                && raw.action == acl.action
                && raw.topic == acl.topic
                && raw.ip == acl.ip
            {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_dir_all;
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };

    use crate::storage::{
        mqtt::acl::AclStorage,
        rocksdb::{column_family_list, RocksDBEngine},
    };

    #[tokio::test]
    async fn acl_storage_test() {
        let config = placement_center_test_conf();
        let rs = Arc::new(RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let acl_storage = AclStorage::new(rs);
        let cluster_name = "test_cluster".to_string();

        let resource_type = MqttAclResourceType::User;
        let resource_name = "test_resource".to_string();
        let topic = "test_topic".to_string();
        let ip = "localhost".to_string();
        let action = MqttAclAction::PubSub;
        let permission = MqttAclPermission::Allow;

        let acl = MqttAcl {
            resource_type: resource_type.clone(),
            resource_name: resource_name.clone(),
            topic: topic.clone(),
            ip: ip.clone(),
            action: action.clone(),
            permission: permission.clone(),
        };

        acl_storage.save(&cluster_name, acl.clone()).unwrap();

        // save a duplicate acl
        acl_storage.save(&cluster_name, acl.clone()).unwrap();

        let res = acl_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 1);

        let acl2 = MqttAcl {
            resource_type: resource_type.clone(),
            resource_name: "test_resource2".to_string(),
            topic: "test_topic2".to_string(),
            ip: "localhost2".to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        };

        acl_storage.save(&cluster_name, acl2.clone()).unwrap();

        let res = acl_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 2);

        let res = acl_storage
            .get(
                &cluster_name,
                resource_type.to_string().as_str(),
                &resource_name,
            )
            .unwrap();

        assert_eq!(res.len(), 1);

        acl_storage.delete(&cluster_name, &acl).unwrap();

        let res = acl_storage
            .get(
                &cluster_name,
                resource_type.to_string().as_str(),
                &resource_name,
            )
            .unwrap();

        assert_eq!(res.len(), 0);

        let res = acl_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 1);

        acl_storage.delete(&cluster_name, &acl2).unwrap();
        let res = acl_storage
            .get(
                &cluster_name,
                resource_type.to_string().as_str(),
                "test_resource2",
            )
            .unwrap();

        assert_eq!(res.len(), 0);

        let res = acl_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 0);

        remove_dir_all(config.rocksdb.data_path).unwrap();
    }
}
