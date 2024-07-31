use crate::{
    cache::placement::PlacementCacheManager,
    storage::{
        keys::{storage_key_mqtt_node_sub_group_leader, storage_key_mqtt_sub_group_leader},
        placement::kv::KvStorage,
        rocksdb::RocksDBEngine,
    },
};
use common_base::errors::RobustMQError;
use std::{cmp::min, collections::HashMap, hash::Hash, sync::Arc};

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

        if let Some(data) = kv_storage.get(key) {
            match data.parse::<u64>() {
                Ok(da) => {
                    if let Some(node_list) = self.cluster_cache.node_list.get(cluster_name) {
                        if node_list.contains_key(&da) {
                            return Ok(da);
                        }
                    }
                }
                Err(_) => {}
            }
        }

        return self.calc_leader();
    }

    fn calc_leader(
        &self,
        cluster_name: &String,
        group_name: &String,
    ) -> Result<u64, RobustMQError> {
        let kv_storage = KvStorage::new(self.rocksdb_engine_handler);
        let key = storage_key_mqtt_node_sub_group_leader(cluster_name);
        let node_leader_data = if let Some(data) = kv_storage.get(key) {
            let data = match serde_json::from_str::<HashMap<u64, Vec<String>>>(&data) {
                Ok(da) => da,
                Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
            };
            data
        } else {
            HashMap::new()
        };

        let mut broker_ids = Vec::new();
        if let Some(cluster) = self.cluster_cache.node_list.get(cluster_name) {
            for (id, _) in cluster.clone() {
                broker_ids.push(id);
            }
        }

        broker_ids.sort();

        let node_sub_info = match self.read_node_sub_info(cluster_name, group_name) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        let mut target_id = 0;
        let mut cur_len = 0;
        for broker_id in broker_ids {
            let size = if let Some(list) = node_sub_info.get(&broker_id) {
                list.len()
            } else {
                0
            };
            if target_id == 0 {
                target_id = broker_id;
                cur_len = size;
                continue;
            }
        }
        return Err(RobustMQError::ClusterNoAvailableNode);
    }

    fn read_node_sub_info(
        &self,
        cluster_name: &String,
        group_name: &String,
    ) -> Result<HashMap<u64, Vec<String>>, RobustMQError> {
        let kv_storage = KvStorage::new(self.rocksdb_engine_handler);
        let key = storage_key_mqtt_node_sub_group_leader(cluster_name);
        if let Some(data) = kv_storage.get(key) {
            match serde_json::from_str::<HashMap<u64, Vec<String>>>(&data) {
                Ok(data) => {
                    return Ok(data);
                }
                Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
            }
        }
        return Ok(HashMap::new());
    }
}
pub fn calc_share_sub_leader(
    cluster_name: &String,
    group_name: &String,
    cluster_cache: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
) -> Result<u64, RobustMQError> {
    let mut broker_ids = Vec::new();
    if let Some(cluster) = cluster_cache.node_list.get(cluster_name) {
        for (id, _) in cluster.clone() {
            broker_ids.push(id);
        }
    }
    broker_ids.sort();
    if broker_ids.len() == 0 {
        return Err(RobustMQError::ClusterDoesNotExist(cluster_name.clone()));
    }
    return Ok(broker_ids.first().unwrap().clone());
}

fn get_node_sub_info(cluster_name: &String, rocksdb_engine_handler: Arc<RocksDBEngine>) {}

fn save_node_sub_info(
    cluster_name: &String,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    group_name: &String,
    node_id: u64,
) -> Result<(), RobustMQError> {
    let kv_storage = KvStorage::new(rocksdb_engine_handler);
    let key = storage_key_mqtt_node_sub_group_leader(cluster_name);
    if let Some(data) = kv_storage.get(key) {
        match serde_json::from_str::<HashMap<u64, Vec<String>>>(&data) {
            Ok(da) => {
                return Ok(());
            }
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        }
    }
    return Ok(());
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
