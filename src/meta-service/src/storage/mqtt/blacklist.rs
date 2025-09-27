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

use crate::storage::engine_meta::{
    engine_delete_by_cluster, engine_prefix_list_by_cluster, engine_save_by_meta,
};
use crate::storage::keys::{storage_key_mqtt_blacklist, storage_key_mqtt_blacklist_prefix};
use rocksdb_engine::RocksDBEngine;

pub struct MqttBlackListStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttBlackListStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttBlackListStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, cluster_name: &str, blacklist: MqttAclBlackList) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(
            cluster_name,
            &blacklist.blacklist_type.to_string(),
            &blacklist.resource_name,
        );
        engine_save_by_meta(self.rocksdb_engine_handler.clone(), key, blacklist)
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<MqttAclBlackList>, CommonError> {
        let prefix_key = storage_key_mqtt_blacklist_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let blacklist = serde_json::from_str::<MqttAclBlackList>(&raw.data)?;
            results.push(blacklist);
        }
        Ok(results)
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        blacklist_type: &str,
        resource_name: &str,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(cluster_name, blacklist_type, resource_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }
}

#[cfg(test)]
mod tests {
    use broker_core::rocksdb::column_family_list;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::utils::file_utils::test_temp_dir;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
    use rocksdb_engine::RocksDBEngine;
    use std::sync::Arc;

    use crate::storage::mqtt::blacklist::MqttBlackListStorage;

    #[tokio::test]
    async fn blacklist_storage_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        let rs = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let blacklist_storage = MqttBlackListStorage::new(rs);
        let cluster_name = "test_cluster".to_string();

        let blacklist1 = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientId,
            resource_name: "resource1".to_string(),
            end_time: 171456001,
            desc: "user1".to_string(),
        };
        let blacklist2 = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: "resource2".to_string(),
            end_time: 171456002,
            desc: "user2".to_string(),
        };

        blacklist_storage
            .save(&cluster_name, blacklist1.clone())
            .unwrap();
        blacklist_storage
            .save(&cluster_name, blacklist2.clone())
            .unwrap();

        let res = blacklist_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 2);

        let res = blacklist_storage
            .list(&cluster_name)
            .unwrap()
            .into_iter()
            .find(|b| {
                b.blacklist_type == blacklist1.blacklist_type
                    && b.resource_name == blacklist1.resource_name
            });
        assert!(res.is_some());

        blacklist_storage
            .delete(
                &cluster_name,
                &blacklist1.blacklist_type.to_string(),
                &blacklist1.resource_name,
            )
            .unwrap();

        let res = blacklist_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 1);
        assert!(res
            .iter()
            .all(|b| b.blacklist_type != blacklist1.blacklist_type
                || b.resource_name != blacklist1.resource_name));
    }
}
