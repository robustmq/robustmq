// Copyright 2023 RobustMQ Team
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

use crate::{
    cache::placement::PlacementCacheManager,
    storage::{
        keys::storage_key_mqtt_node_sub_group_leader, placement::kv::KvStorage,
        rocksdb::RocksDBEngine,
    },
};
use common_base::error::common::CommonError;
use std::{collections::HashMap, sync::Arc};

pub struct ShareSubLeader {
    cluster_cache: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShareSubLeader {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        return ShareSubLeader {
            cluster_cache,
            rocksdb_engine_handler,
        };
    }

    pub fn get_leader_node(
        &self,
        cluster_name: &String,
        group_name: &String,
    ) -> Result<u64, CommonError> {
        let mut broker_ids = Vec::new();
        if let Some(cluster) = self.cluster_cache.node_list.get(cluster_name) {
            for (id, _) in cluster.clone() {
                broker_ids.push(id);
            }
        }

        broker_ids.sort();

        let node_sub_info = match self.read_node_sub_info(cluster_name) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        for (broker_id, group_list) in node_sub_info.clone() {
            if group_list.contains(group_name) {
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

        self.save_node_sub_info(cluster_name, target_broker_id, group_name)?;
        return Ok(target_broker_id);
    }

    #[allow(dead_code)]
    pub fn remove_group_by_node(
        &self,
        cluster_name: &String,
        group_name: &String,
    ) -> Result<(), CommonError> {
        let mut node_sub_info = match self.read_node_sub_info(cluster_name) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };
        for (broker_id, mut group_list) in node_sub_info.clone() {
            if group_list.contains(group_name) {
                group_list.retain(|x| *x != group_name.to_string());
                node_sub_info.insert(broker_id, group_list);

                let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
                let key = storage_key_mqtt_node_sub_group_leader(cluster_name);
                match serde_json::to_string(&node_sub_info) {
                    Ok(value) => match kv_storage.set(key, value) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e);
                        }
                    },
                    Err(e) => {
                        return Err(CommonError::CommmonError(e.to_string()));
                    }
                }
                break;
            }
        }
        return Ok(());
    }

    pub fn delete_node(&self, cluster_name: &String, broker_id: u64) -> Result<(), CommonError> {
        let mut node_sub_info = match self.read_node_sub_info(cluster_name) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };
        if node_sub_info.contains_key(&broker_id) {
            node_sub_info.remove(&broker_id);
            let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
            let key = storage_key_mqtt_node_sub_group_leader(cluster_name);

            match serde_json::to_string(&node_sub_info) {
                Ok(value) => match kv_storage.set(key, value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
                Err(e) => {
                    return Err(CommonError::CommmonError(e.to_string()));
                }
            }
        }
        return Ok(());
    }

    fn save_node_sub_info(
        &self,
        cluster_name: &String,
        broker_id: u64,
        group_name: &String,
    ) -> Result<(), CommonError> {
        let mut node_sub_info = match self.read_node_sub_info(cluster_name) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };
        if let Some(data) = node_sub_info.get_mut(&broker_id) {
            if !data.contains(group_name) {
                data.push(group_name.clone());
            }
        } else {
            node_sub_info.insert(broker_id, vec![group_name.clone()]);
        }

        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let key = storage_key_mqtt_node_sub_group_leader(cluster_name);

        match serde_json::to_string(&node_sub_info) {
            Ok(value) => match kv_storage.set(key, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e);
                }
            },
            Err(e) => {
                return Err(CommonError::CommmonError(e.to_string()));
            }
        }
        return Ok(());
    }

    fn read_node_sub_info(
        &self,
        cluster_name: &String,
    ) -> Result<HashMap<u64, Vec<String>>, CommonError> {
        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let key = storage_key_mqtt_node_sub_group_leader(cluster_name);
        match kv_storage.get(key) {
            Ok(Some(data)) => match serde_json::from_str::<HashMap<u64, Vec<String>>>(&data) {
                Ok(data) => {
                    return Ok(data);
                }
                Err(e) => return Err(CommonError::CommmonError(e.to_string())),
            },
            Ok(None) => {}
            Err(e) => {
                return Err(e);
            }
        }

        return Ok(HashMap::new());
    }
}

#[cfg(test)]
mod tests {
    use super::ShareSubLeader;
    use crate::{cache::placement::PlacementCacheManager, storage::rocksdb::RocksDBEngine};
    use common_base::{
        config::placement_center::PlacementCenterConfig,
        tools::{now_mills, unique_id},
    };
    use metadata_struct::placement::broker_node::BrokerNode;
    use protocol::placement_center::generate::common::ClusterType;
    use std::sync::Arc;

    #[test]
    fn node_leader_info_test() {
        let config = PlacementCenterConfig::default();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(&config));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let share_sub = ShareSubLeader::new(cluster_cache, rocksdb_engine_handler.clone());
        let cluster_name = unique_id();
        let broker_id = 1;
        let group_name = "group1".to_string();
        share_sub
            .save_node_sub_info(&cluster_name, broker_id, &group_name)
            .unwrap();
        let result = share_sub.read_node_sub_info(&cluster_name).unwrap();
        assert!(result.contains_key(&broker_id));
        assert!(result.get(&broker_id).unwrap().contains(&group_name));

        let group_name2 = "group2".to_string();
        share_sub
            .save_node_sub_info(&cluster_name, broker_id, &group_name2)
            .unwrap();
        let result = share_sub.read_node_sub_info(&cluster_name).unwrap();
        assert!(result.contains_key(&broker_id));
        assert!(result.get(&broker_id).unwrap().contains(&group_name2));
        assert!(result.get(&broker_id).unwrap().contains(&group_name));

        let broker_id = 2;
        let group_name3 = "test".to_string();
        share_sub
            .save_node_sub_info(&cluster_name, broker_id, &group_name3)
            .unwrap();
        let result = share_sub.read_node_sub_info(&cluster_name).unwrap();
        assert!(result.contains_key(&broker_id));
        assert!(result.get(&broker_id).unwrap().contains(&group_name3));

        share_sub
            .remove_group_by_node(&cluster_name, &group_name3)
            .unwrap();
        let result = share_sub.read_node_sub_info(&cluster_name).unwrap();
        assert!(!result.get(&broker_id).unwrap().contains(&group_name3));

        share_sub.delete_node(&cluster_name, broker_id).unwrap();
        let result = share_sub.read_node_sub_info(&cluster_name).unwrap();
        assert!(!result.contains_key(&broker_id));
    }

    #[test]
    fn get_leader_node_test() {
        let config = PlacementCenterConfig::default();
        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(&config));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        cluster_cache.add_node(BrokerNode {
            cluster_name: cluster_name.clone(),
            cluster_type: ClusterType::MqttBrokerServer.as_str_name().to_string(),
            node_id: 1,
            node_ip: "".to_string(),
            node_inner_addr: "".to_string(),
            extend: "".to_string(),
            create_time: now_mills(),
        });
        cluster_cache.add_node(BrokerNode {
            cluster_name: cluster_name.clone(),
            cluster_type: ClusterType::MqttBrokerServer.as_str_name().to_string(),
            node_id: 2,
            node_ip: "".to_string(),
            node_inner_addr: "".to_string(),
            extend: "".to_string(),
            create_time: now_mills(),
        });
        cluster_cache.add_node(BrokerNode {
            cluster_name: cluster_name.clone(),
            cluster_type: ClusterType::MqttBrokerServer.as_str_name().to_string(),
            node_id: 3,
            node_ip: "".to_string(),
            node_inner_addr: "".to_string(),
            extend: "".to_string(),
            create_time: now_mills(),
        });

        let share_sub = ShareSubLeader::new(cluster_cache, rocksdb_engine_handler.clone());

        let group_name = "group1".to_string();
        let node = share_sub
            .get_leader_node(&cluster_name, &group_name)
            .unwrap();
        assert_eq!(node, 1);
        let node = share_sub
            .get_leader_node(&cluster_name, &group_name)
            .unwrap();
        assert_eq!(node, 1);

        let group_name = "group2".to_string();
        let node = share_sub
            .get_leader_node(&cluster_name, &group_name)
            .unwrap();
        assert_eq!(node, 2);

        let group_name = "group3".to_string();
        let node = share_sub
            .get_leader_node(&cluster_name, &group_name)
            .unwrap();
        assert_eq!(node, 3);

        let group_name = "group4".to_string();
        let node = share_sub
            .get_leader_node(&cluster_name, &group_name)
            .unwrap();
        assert_eq!(node, 1);
    }
}
