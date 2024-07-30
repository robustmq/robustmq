use crate::{
    cache::placement::PlacementCacheManager,
    storage::{
        keys::storage_key_mqtt_sub_group_leader, placement::kv::KvStorage, rocksdb::RocksDBEngine,
    },
};
use common_base::errors::RobustMQError;
use std::sync::Arc;

pub fn calc_share_sub_leader(
    cluster_name: &String,
    group_name: &String,
    cluster_cache: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
) -> Result<u64, RobustMQError> {
    let key = storage_key_mqtt_sub_group_leader(cluster_name, group_name);
    let kv_storage = KvStorage::new(rocksdb_engine_handler);

    if let Some(data) = kv_storage.get(key) {
        match data.parse::<u64>() {
            Ok(da) => {
                if let Some(node_list) = cluster_cache.node_list.get(cluster_name) {
                    if node_list.contains_key(&da) {
                        return Ok(da);
                    }
                }
            }
            Err(_) => {}
        }
    }

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

#[cfg(test)]
mod tests {
    #[test]
    fn calc_share_sub_leader_test() {
        let cluster_name = "test".to_string();
        let group_name = "group1".to_string();
    }
}
