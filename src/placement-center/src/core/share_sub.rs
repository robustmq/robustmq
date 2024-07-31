use crate::{
    cache::placement::PlacementCacheManager,
    storage::{
        keys::{storage_key_mqtt_node_sub_group_leader, storage_key_mqtt_sub_group_leader},
        placement::kv::KvStorage,
        rocksdb::RocksDBEngine,
    },
};
use common_base::errors::RobustMQError;
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
    ) -> Result<u64, RobustMQError> {
        let key = storage_key_mqtt_sub_group_leader(cluster_name, group_name);
        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());

        match kv_storage.get(key) {
            Ok(Some(data)) => match data.parse::<u64>() {
                Ok(da) => {
                    if let Some(node_list) = self.cluster_cache.node_list.get(cluster_name) {
                        if node_list.contains_key(&da) {
                            return Ok(da);
                        }
                    }
                }
                Err(_) => {}
            },
            Ok(None) => {}
            Err(e) => {}
        }

        return self.calc_leader(cluster_name, group_name);
    }

    fn calc_leader(
        &self,
        cluster_name: &String,
        group_name: &String,
    ) -> Result<u64, RobustMQError> {
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

            if size > cur_len {
                target_broker_id = broker_id;
                cur_len = size;
            }
        }

        if target_broker_id == 0 {
            return Err(RobustMQError::ClusterNoAvailableNode);
        }

        match self.save_node_sub_info(cluster_name, target_broker_id, group_name) {
            Ok(()) => {}
            Err(e) => {
                return Err(e);
            }
        }
        return Ok(target_broker_id);
    }

    fn save_node_sub_info(
        &self,
        cluster_name: &String,
        broker_id: u64,
        group_name: &String,
    ) -> Result<(), RobustMQError> {
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
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
        return Ok(());
    }

    fn read_node_sub_info(
        &self,
        cluster_name: &String,
    ) -> Result<HashMap<u64, Vec<String>>, RobustMQError> {
        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let key = storage_key_mqtt_node_sub_group_leader(cluster_name);
        match kv_storage.get(key) {
            Ok(Some(data)) => match serde_json::from_str::<HashMap<u64, Vec<String>>>(&data) {
                Ok(data) => {
                    return Ok(data);
                }
                Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
            },
            Ok(None) => {}
            Err(e) => {}
        }

        return Ok(HashMap::new());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::{config::placement_center::PlacementCenterConfig, tools::now_mills};
    use metadata_struct::placement::broker_node::BrokerNode;
    use protocol::placement_center::generate::common::ClusterType;

    use crate::{cache::placement::PlacementCacheManager, storage::rocksdb::RocksDBEngine};

    #[test]
    fn calc_share_sub_leader_test() {
        let config = PlacementCenterConfig::default();
        let cluster_name = "test".to_string();
        let group_name = "group1".to_string();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(&config));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler));
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
    }
}
