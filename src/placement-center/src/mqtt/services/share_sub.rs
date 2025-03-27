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

use crate::core::cache::PlacementCacheManager;
use crate::server::grpc::validate::ValidateExt;
use crate::storage::keys::storage_key_mqtt_node_sub_group_leader;
use crate::storage::placement::kv::KvStorage;
use crate::storage::rocksdb::RocksDBEngine;
use common_base::error::common::CommonError;
use protocol::placement_center::placement_center_mqtt::{
    GetShareSubLeaderReply, GetShareSubLeaderRequest,
};
use std::collections::HashMap;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct ShareSubLeader {
    cluster_cache: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShareSubLeader {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        ShareSubLeader {
            cluster_cache,
            rocksdb_engine_handler,
        }
    }

    pub fn get_leader_node(
        &self,
        cluster_name: &str,
        group_name: &String,
    ) -> Result<u64, CommonError> {
        let mut broker_ids = self
            .cluster_cache
            .get_broker_node_id_by_cluster(cluster_name);

        broker_ids.sort();

        let node_sub_info = self.read_node_sub_info(cluster_name)?;

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
        Ok(target_broker_id)
    }

    #[allow(dead_code)]
    pub fn remove_group_by_node(
        &self,
        cluster_name: &str,
        group_name: &String,
    ) -> Result<(), CommonError> {
        let mut node_sub_info = self.read_node_sub_info(cluster_name)?;
        for (broker_id, mut group_list) in node_sub_info.clone() {
            if group_list.contains(group_name) {
                group_list.retain(|x| x != group_name);
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
                        return Err(CommonError::CommonError(e.to_string()));
                    }
                }
                break;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_node(&self, cluster_name: &str, broker_id: u64) -> Result<(), CommonError> {
        let mut node_sub_info = self.read_node_sub_info(cluster_name)?;
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
                    return Err(CommonError::CommonError(e.to_string()));
                }
            }
        }
        Ok(())
    }

    fn save_node_sub_info(
        &self,
        cluster_name: &str,
        broker_id: u64,
        group_name: &String,
    ) -> Result<(), CommonError> {
        let mut node_sub_info = self.read_node_sub_info(cluster_name)?;
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
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
        Ok(())
    }

    fn read_node_sub_info(
        &self,
        cluster_name: &str,
    ) -> Result<HashMap<u64, Vec<String>>, CommonError> {
        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let key = storage_key_mqtt_node_sub_group_leader(cluster_name);
        match kv_storage.get(key) {
            Ok(Some(data)) => match serde_json::from_str::<HashMap<u64, Vec<String>>>(&data) {
                Ok(data) => {
                    return Ok(data);
                }
                Err(e) => return Err(CommonError::CommonError(e.to_string())),
            },
            Ok(None) => {}
            Err(e) => {
                return Err(e);
            }
        }

        Ok(HashMap::new())
    }
}

pub fn get_share_sub_leader_by_req(
    cluster_cache: &Arc<PlacementCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<GetShareSubLeaderRequest>,
) -> Result<Response<GetShareSubLeaderReply>, Status> {
    let req = request.into_inner();
    req.validate_ext()?;
    let cluster_name = req.cluster_name;
    let group_name = req.group_name;
    let mut reply = GetShareSubLeaderReply::default();
    let share_sub = ShareSubLeader::new(cluster_cache.clone(), rocksdb_engine_handler.clone());

    let leader_broker = match share_sub.get_leader_node(&cluster_name, &group_name) {
        Ok(data) => data,
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    };

    if let Some(node) = cluster_cache.get_broker_node(&cluster_name, leader_broker) {
        reply.broker_id = leader_broker;
        reply.broker_addr = node.node_inner_addr;
        reply.extend_info = node.extend;
    }

    Ok(Response::new(reply))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::tools::{now_mills, unique_id};
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::placement::node::BrokerNode;
    use protocol::placement_center::placement_center_inner::ClusterType;

    use super::ShareSubLeader;
    use crate::core::cache::PlacementCacheManager;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[test]
    fn node_leader_info_test() {
        let config = placement_center_test_conf();

        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
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
        let config = placement_center_test_conf();

        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        cluster_cache.add_broker_node(BrokerNode {
            cluster_name: cluster_name.clone(),
            cluster_type: ClusterType::MqttBrokerServer.as_str_name().to_string(),
            node_id: 1,
            node_ip: "".to_string(),
            node_inner_addr: "".to_string(),
            extend: "".to_string(),
            create_time: now_mills(),
        });
        cluster_cache.add_broker_node(BrokerNode {
            cluster_name: cluster_name.clone(),
            cluster_type: ClusterType::MqttBrokerServer.as_str_name().to_string(),
            node_id: 2,
            node_ip: "".to_string(),
            node_inner_addr: "".to_string(),
            extend: "".to_string(),
            create_time: now_mills(),
        });
        cluster_cache.add_broker_node(BrokerNode {
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
