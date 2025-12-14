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

use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::storage::common::kv::KvStorage;
use crate::storage::keys::storage_key_mqtt_node_sub_group_leader;
use common_base::error::common::CommonError;
use protocol::meta::meta_service_mqtt::{GetShareSubLeaderReply, GetShareSubLeaderRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ShareSubLeader {
    cache_manager: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShareSubLeader {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        ShareSubLeader {
            cache_manager,
            rocksdb_engine_handler,
        }
    }

    pub fn get_leader_node(&self, group_name: &str) -> Result<u64, CommonError> {
        let mut broker_ids: Vec<u64> = self
            .cache_manager
            .node_list
            .iter()
            .map(|node| node.node_id)
            .collect();

        broker_ids.sort();

        let node_sub_info = self.read_node_sub_info()?;

        for (broker_id, group_list) in node_sub_info.clone() {
            if group_list.iter().any(|g| g == group_name) && broker_ids.contains(&broker_id) {
                return Ok(broker_id);
            }
        }

        let mut target_broker_id = 0;
        let mut cur_len = 0;
        for broker_id in broker_ids {
            let size = if let Some(list) = node_sub_info.get(&broker_id) {
                list.len()
            } else {
                0
            };

            if target_broker_id == 0 {
                target_broker_id = broker_id;
                cur_len = size;
                continue;
            }

            if size < cur_len {
                target_broker_id = broker_id;
                cur_len = size;
            }
        }

        if target_broker_id == 0 {
            return Err(CommonError::ClusterNoAvailableNode);
        }

        self.save_node_sub_info(target_broker_id, group_name)?;
        Ok(target_broker_id)
    }

    #[allow(dead_code)]
    pub fn remove_group_by_node(&self, group_name: &str) -> Result<(), CommonError> {
        let mut node_sub_info = self.read_node_sub_info()?;
        let mut modified = false;

        for (broker_id, group_list) in node_sub_info.clone() {
            if group_list.iter().any(|g| g == group_name) {
                let updated_list: Vec<String> = group_list
                    .into_iter()
                    .filter(|x| x.as_str() != group_name)
                    .collect();
                node_sub_info.insert(broker_id, updated_list);
                modified = true;
                break;
            }
        }

        if modified {
            let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
            let key = storage_key_mqtt_node_sub_group_leader();
            let value = serde_json::to_string(&node_sub_info)
                .map_err(|e| CommonError::CommonError(e.to_string()))?;
            kv_storage.set(key, value)?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_node(&self, broker_id: u64) -> Result<(), CommonError> {
        let mut node_sub_info = self.read_node_sub_info()?;

        if !node_sub_info.contains_key(&broker_id) {
            return Ok(());
        }

        node_sub_info.remove(&broker_id);

        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let key = storage_key_mqtt_node_sub_group_leader();
        let value = serde_json::to_string(&node_sub_info)
            .map_err(|e| CommonError::CommonError(e.to_string()))?;
        kv_storage.set(key, value)?;

        Ok(())
    }

    fn save_node_sub_info(&self, broker_id: u64, group_name: &str) -> Result<(), CommonError> {
        let mut node_sub_info = self.read_node_sub_info()?;

        // Add group to broker's list
        node_sub_info
            .entry(broker_id)
            .or_insert_with(Vec::new)
            .push(group_name.to_owned());

        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let key = storage_key_mqtt_node_sub_group_leader();
        let value = serde_json::to_string(&node_sub_info)
            .map_err(|e| CommonError::CommonError(e.to_string()))?;

        kv_storage.set(key, value)?;

        Ok(())
    }

    fn read_node_sub_info(&self) -> Result<HashMap<u64, Vec<String>>, CommonError> {
        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let key = storage_key_mqtt_node_sub_group_leader();

        let data = match kv_storage.get(key)? {
            Some(data) => data,
            None => return Ok(HashMap::new()),
        };

        serde_json::from_str(&data).map_err(|e| CommonError::CommonError(e.to_string()))
    }
}

pub fn get_share_sub_leader_by_req(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetShareSubLeaderRequest,
) -> Result<GetShareSubLeaderReply, MetaServiceError> {
    let share_sub = ShareSubLeader::new(cache_manager.clone(), rocksdb_engine_handler.clone());

    // Get leader broker ID for the shared subscription group
    let leader_broker = share_sub
        .get_leader_node(&req.group_name)
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))?;

    // Get broker node details from cache
    match cache_manager.get_broker_node(leader_broker) {
        Some(node) => Ok(GetShareSubLeaderReply {
            broker_id: node.node_id,
            broker_addr: node.node_ip,
            extend_info: node.extend,
        }),
        None => Err(MetaServiceError::NoAvailableBrokerNode),
    }
}

#[cfg(test)]
mod tests {
    use super::ShareSubLeader;
    use crate::core::cache::CacheManager;
    use common_base::tools::{now_second, unique_id};
    use common_base::utils::file_utils::test_temp_dir;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use metadata_struct::meta::node::BrokerNode;
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use rocksdb_engine::storage::family::column_family_list;
    use std::sync::Arc;

    #[test]
    fn node_leader_info_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());

        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let cluster_cache = Arc::new(CacheManager::new(rocksdb_engine_handler.clone()));
        let share_sub = ShareSubLeader::new(cluster_cache, rocksdb_engine_handler.clone());
        let broker_id = 1;
        let group_name = "group1".to_string();
        share_sub
            .save_node_sub_info(broker_id, &group_name)
            .unwrap();
        let result = share_sub.read_node_sub_info().unwrap();
        assert!(result.contains_key(&broker_id));
        assert!(result.get(&broker_id).unwrap().contains(&group_name));

        let group_name2 = "group2".to_string();
        share_sub
            .save_node_sub_info(broker_id, &group_name2)
            .unwrap();
        let result = share_sub.read_node_sub_info().unwrap();
        assert!(result.contains_key(&broker_id));
        assert!(result.get(&broker_id).unwrap().contains(&group_name2));
        assert!(result.get(&broker_id).unwrap().contains(&group_name));

        let broker_id = 2;
        let group_name3 = "test".to_string();
        share_sub
            .save_node_sub_info(broker_id, &group_name3)
            .unwrap();
        let result = share_sub.read_node_sub_info().unwrap();
        assert!(result.contains_key(&broker_id));
        assert!(result.get(&broker_id).unwrap().contains(&group_name3));

        share_sub.remove_group_by_node(&group_name3).unwrap();
        let result = share_sub.read_node_sub_info().unwrap();
        assert!(!result.get(&broker_id).unwrap().contains(&group_name3));

        share_sub.delete_node(broker_id).unwrap();
        let result = share_sub.read_node_sub_info().unwrap();
        assert!(!result.contains_key(&broker_id));
    }

    #[test]
    fn get_leader_node_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let cluster_cache = Arc::new(CacheManager::new(rocksdb_engine_handler.clone()));
        cluster_cache.add_broker_node(BrokerNode {
            roles: Vec::new(),
            node_id: 1,
            node_ip: "".to_string(),
            node_inner_addr: "".to_string(),
            extend: Vec::new(),
            start_time: now_second(),
            register_time: now_second(),
        });
        cluster_cache.add_broker_node(BrokerNode {
            roles: Vec::new(),
            node_id: 2,
            node_ip: "".to_string(),
            node_inner_addr: "".to_string(),
            extend: Vec::new(),
            start_time: now_second(),
            register_time: now_second(),
        });
        cluster_cache.add_broker_node(BrokerNode {
            roles: Vec::new(),
            node_id: 3,
            node_ip: "".to_string(),
            node_inner_addr: "".to_string(),
            extend: Vec::new(),
            start_time: now_second(),
            register_time: now_second(),
        });

        let share_sub = ShareSubLeader::new(cluster_cache, rocksdb_engine_handler.clone());

        let group_name = "group1".to_string();
        let node = share_sub.get_leader_node(&group_name).unwrap();
        assert_eq!(node, 1);

        let node = share_sub.get_leader_node(&group_name).unwrap();
        assert_eq!(node, 1);

        let group_name = "group2".to_string();
        let node = share_sub.get_leader_node(&group_name).unwrap();
        assert_eq!(node, 2);

        let group_name = "group3".to_string();
        let node = share_sub.get_leader_node(&group_name).unwrap();
        assert_eq!(node, 3);

        let group_name = "group3".to_string();
        let node = share_sub.get_leader_node(&group_name).unwrap();
        assert_eq!(node, 3);

        let group_name = "group4".to_string();
        let node = share_sub.get_leader_node(&group_name).unwrap();
        assert_eq!(node, 1);

        let group_name = "group4".to_string();
        let node = share_sub.get_leader_node(&group_name).unwrap();
        assert_eq!(node, 1);
    }
}
