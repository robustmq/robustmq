use crate::{
    cache::cluster::ClusterCache,
    storage::{
        placement::{cluster::ClusterStorage, config::ResourceConfigStorage, node::NodeStorage},
        rocksdb::RocksDBEngine,
    },
};
use common_base::{
    errors::RobustMQError,
    tools::{now_mills, unique_id},
};
use metadata_struct::placement::{broker_node::BrokerNode, cluster::ClusterInfo};
use prost::Message as _;
use protocol::placement_center::generate::placement::{
    DeleteResourceConfigRequest, RegisterNodeRequest, SetResourceConfigRequest,
    UnRegisterNodeRequest,
};
use std::sync::Arc;
use tonic::Status;

pub struct DataRouteCluster {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cluster_cache: Arc<ClusterCache>,
}

impl DataRouteCluster {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cluster_cache: Arc<ClusterCache>,
    ) -> Self {
        return DataRouteCluster {
            rocksdb_engine_handler,
            cluster_cache,
        };
    }

    pub fn add_node(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: RegisterNodeRequest = RegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_type = req.cluster_type();
        let cluster_name = req.cluster_name;
        let node = BrokerNode {
            node_id: req.node_id,
            node_ip: req.node_ip,
            node_inner_addr: req.node_inner_addr,
            extend: req.extend_info,
            cluster_name: cluster_name.clone(),
            cluster_type: cluster_type.as_str_name().to_string(),
            create_time: now_mills(),
        };

        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        let cluster_storage = ClusterStorage::new(self.rocksdb_engine_handler.clone());

        // update cluster
        if !self.cluster_cache.cluster_list.contains_key(&cluster_name) {
            let cluster_info = ClusterInfo {
                cluster_uid: unique_id(),
                cluster_name: cluster_name.clone(),
                cluster_type: cluster_type.as_str_name().to_string(),
                create_time: now_mills(),
            };
            self.cluster_cache.add_cluster(cluster_info.clone());
            match cluster_storage.save(cluster_info) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // update node
        self.cluster_cache.add_node(node.clone());
        return node_storage.save(cluster_name.clone(), node);
    }

    pub fn delete_node(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_name = req.cluster_name;
        let node_id = req.node_id;
        self.cluster_cache
            .remove_node(cluster_name.clone(), node_id);

        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        return node_storage.delete(cluster_name.clone(), node_id);
    }

    pub fn set_resource_config(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = SetResourceConfigRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let config_storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        return config_storage.save(req.cluster_name, req.resources, req.config);
    }

    pub fn delete_resource_config(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = DeleteResourceConfigRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let config_storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        return config_storage.delete(req.cluster_name, req.resources);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        cache::cluster::ClusterCache,
        raft::route::cluster::DataRouteCluster,
        storage::{
            placement::cluster::ClusterStorage, placement::node::NodeStorage,
            rocksdb::RocksDBEngine,
        },
    };
    use common_base::config::placement_center::PlacementCenterConfig;
    use prost::Message as _;
    use protocol::placement_center::generate::{
        common::ClusterType, placement::RegisterNodeRequest,
    };

    #[test]
    fn register_unregister_node() {
        let cluster_name = "test-cluster".to_string();
        let node_id = 1;
        let node_ip = "127.0.0.1".to_string();

        let mut req = RegisterNodeRequest::default();
        req.node_id = node_id;
        req.node_ip = node_ip.clone();
        req.cluster_type = ClusterType::MqttBrokerServer.into();
        req.cluster_name = cluster_name.clone();
        req.extend_info = "{}".to_string();
        let data = RegisterNodeRequest::encode_to_vec(&req);
        let rocksdb_engine = Arc::new(RocksDBEngine::new(&PlacementCenterConfig::default()));
        let cluster_cache = Arc::new(ClusterCache::new());

        let route = DataRouteCluster::new(rocksdb_engine.clone(), cluster_cache);
        let _ = route.add_node(data);

        let node_storage = NodeStorage::new(rocksdb_engine.clone());
        let cluster_storage = ClusterStorage::new(rocksdb_engine.clone());

        let cluster = cluster_storage.get(cluster_name.clone()).unwrap();
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);

        let node = node_storage.get(cluster_name.clone(), node_id).unwrap();
        let nd = node.unwrap();
        assert_eq!(nd.node_id, node_id);
        assert_eq!(nd.node_ip, node_ip);

        let _ = node_storage.delete(cluster_name.clone(), node_id);
        let res = node_storage.get(cluster_name.clone(), node_id).unwrap();
        assert!(res.is_none());

        let cluster = cluster_storage.get(cluster_name.clone()).unwrap();
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
    }
}
