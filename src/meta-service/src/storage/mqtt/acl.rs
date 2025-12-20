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

use crate::storage::keys::{storage_key_mqtt_acl, storage_key_mqtt_acl_prefix};
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
        let mut acl_list = self.get(&resource_type_str, &acl.resource_name)?;

        if self.acl_exists(&acl_list, &acl) {
            return Ok(());
        }

        acl_list.push(acl.clone());

        let key = storage_key_mqtt_acl(&resource_type_str, &acl.resource_name);
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

    pub fn delete(&self, delete_acl: &MqttAcl) -> Result<(), CommonError> {
        let resource_type_str = delete_acl.resource_type.to_string();
        let acl_list = self.get(&resource_type_str, &delete_acl.resource_name)?;

        if !self.acl_exists(&acl_list, delete_acl) {
            return Ok(());
        }

        let new_acl_list: Vec<MqttAcl> = acl_list
            .into_iter()
            .filter(|raw| !Self::acl_matches(raw, delete_acl))
            .collect();

        let key = storage_key_mqtt_acl(&resource_type_str, &delete_acl.resource_name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, new_acl_list)
    }

    pub fn get(
        &self,
        resource_type: &str,
        resource_name: &str,
    ) -> Result<Vec<MqttAcl>, CommonError> {
        let key = storage_key_mqtt_acl(resource_type, resource_name);
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

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
    use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
    use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> AclStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        AclStorage::new(test_rocksdb_instance())
    }

    fn create_acl(
        resource_type: MqttAclResourceType,
        resource_name: &str,
        topic: &str,
        permission: MqttAclPermission,
        action: MqttAclAction,
    ) -> MqttAcl {
        MqttAcl {
            resource_type,
            resource_name: resource_name.to_string(),
            topic: topic.to_string(),
            ip: "0.0.0.0".to_string(),
            action,
            permission,
        }
    }

    #[test]
    fn test_acl_crud() {
        let storage = setup_storage();

        // Save & Get
        let acl = create_acl(
            MqttAclResourceType::User,
            "alice",
            "sensor/#",
            MqttAclPermission::Allow,
            MqttAclAction::Subscribe,
        );
        storage.save(acl.clone()).unwrap();

        let acls = storage
            .get(&acl.resource_type.to_string(), &acl.resource_name)
            .unwrap();
        assert_eq!(acls.len(), 1);

        // List
        let acl2 = create_acl(
            MqttAclResourceType::User,
            "bob",
            "admin/#",
            MqttAclPermission::Deny,
            MqttAclAction::Publish,
        );
        storage.save(acl2).unwrap();
        assert_eq!(storage.list_all().unwrap().len(), 2);

        // Delete
        storage.delete(&acl).unwrap();
        let acls = storage
            .get(&acl.resource_type.to_string(), &acl.resource_name)
            .unwrap();
        assert_eq!(acls.len(), 0);
    }

    #[test]
    fn test_duplicate_acl() {
        let storage = setup_storage();

        let acl = create_acl(
            MqttAclResourceType::User,
            "user1",
            "topic1",
            MqttAclPermission::Allow,
            MqttAclAction::PubSub,
        );

        // Save twice
        storage.save(acl.clone()).unwrap();
        storage.save(acl.clone()).unwrap();

        // Should only have one
        let acls = storage
            .get(&acl.resource_type.to_string(), &acl.resource_name)
            .unwrap();
        assert_eq!(acls.len(), 1);
    }

    #[test]
    fn test_multiple_acls_per_resource() {
        let storage = setup_storage();

        // Same resource, different topics
        let acl1 = create_acl(
            MqttAclResourceType::User,
            "alice",
            "sensor/#",
            MqttAclPermission::Allow,
            MqttAclAction::Subscribe,
        );
        let acl2 = create_acl(
            MqttAclResourceType::User,
            "alice",
            "device/#",
            MqttAclPermission::Allow,
            MqttAclAction::Publish,
        );

        storage.save(acl1.clone()).unwrap();
        storage.save(acl2.clone()).unwrap();

        let acls = storage.get("User", "alice").unwrap();
        assert_eq!(acls.len(), 2);

        // Delete one
        storage.delete(&acl1).unwrap();
        let acls = storage.get("User", "alice").unwrap();
        assert_eq!(acls.len(), 1);
        assert_eq!(acls[0].topic, "device/#");
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        let acls = storage.get("User", "nonexistent").unwrap();
        assert!(acls.is_empty());
    }
}
