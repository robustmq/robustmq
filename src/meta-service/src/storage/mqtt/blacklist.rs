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
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;

use rocksdb_engine::keys::meta::{storage_key_mqtt_blacklist, storage_key_mqtt_blacklist_prefix};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_prefix_list_by_meta_metadata,
    engine_save_by_meta_metadata,
};

pub struct MqttBlackListStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttBlackListStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttBlackListStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, blacklist: MqttAclBlackList) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(
            &blacklist.blacklist_type.to_string(),
            &blacklist.resource_name,
        );
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, blacklist)
    }

    pub fn list_all(&self) -> Result<Vec<MqttAclBlackList>, CommonError> {
        let prefix_key = storage_key_mqtt_blacklist_prefix();
        let data = engine_prefix_list_by_meta_metadata::<MqttAclBlackList>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, blacklist_type: &str, resource_name: &str) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(blacklist_type, resource_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::tools::now_second;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttBlackListStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        MqttBlackListStorage::new(test_rocksdb_instance())
    }

    fn create_blacklist(
        blacklist_type: MqttAclBlackListType,
        resource_name: &str,
    ) -> MqttAclBlackList {
        MqttAclBlackList {
            blacklist_type,
            resource_name: resource_name.to_string(),
            end_time: now_second() + 3600,
            desc: format!("blocked: {}", resource_name),
        }
    }

    #[test]
    fn test_blacklist_crud() {
        let storage = setup_storage();

        // Save & List
        let bl1 = create_blacklist(MqttAclBlackListType::ClientId, "client_blocked");
        storage.save(bl1.clone()).unwrap();

        let list = storage.list_all().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].resource_name, "client_blocked");

        // Save another
        let bl2 = create_blacklist(MqttAclBlackListType::User, "user_blocked");
        storage.save(bl2.clone()).unwrap();
        assert_eq!(storage.list_all().unwrap().len(), 2);

        // Delete & Verify
        storage
            .delete(&bl1.blacklist_type.to_string(), &bl1.resource_name)
            .unwrap();

        let remaining = storage.list_all().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].resource_name, "user_blocked");
    }

    #[test]
    fn test_blacklist_types() {
        let storage = setup_storage();

        // Test different blacklist types
        let client_id_bl = create_blacklist(MqttAclBlackListType::ClientId, "client1");
        let user_bl = create_blacklist(MqttAclBlackListType::User, "user1");
        let ip_bl = create_blacklist(MqttAclBlackListType::Ip, "192.168.1.100");

        storage.save(client_id_bl.clone()).unwrap();
        storage.save(user_bl.clone()).unwrap();
        storage.save(ip_bl.clone()).unwrap();

        let list = storage.list_all().unwrap();
        assert_eq!(list.len(), 3);

        // Verify each type exists
        assert!(list
            .iter()
            .any(|b| b.blacklist_type == MqttAclBlackListType::ClientId));
        assert!(list
            .iter()
            .any(|b| b.blacklist_type == MqttAclBlackListType::User));
        assert!(list
            .iter()
            .any(|b| b.blacklist_type == MqttAclBlackListType::Ip));
    }

    #[test]
    fn test_update_blacklist() {
        let storage = setup_storage();

        // Save initial
        let initial = create_blacklist(MqttAclBlackListType::ClientId, "client1");
        storage.save(initial.clone()).unwrap();

        // Update with different desc
        let mut updated = initial.clone();
        updated.desc = "updated description".to_string();
        storage.save(updated).unwrap();

        let list = storage.list_all().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].desc, "updated description");
    }

    #[test]
    fn test_empty_list() {
        let storage = setup_storage();
        let list = storage.list_all().unwrap();
        assert!(list.is_empty());
    }
}
