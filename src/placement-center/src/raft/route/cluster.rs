use crate::{
    cache::cluster::ClusterCache,
    storage::{cluster::cluster::ClusterStorage, cluster::node::NodeStorage, rocksdb::RocksDBEngine},
    structs::{cluster::ClusterInfo, node::Node},
};
use common_base::{
    errors::RobustMQError,
    tools::{now_mills, unique_id},
};
use prost::Message as _;
use protocol::placement_center::generate::placement::{RegisterNodeRequest, UnRegisterNodeRequest};
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

    //BrokerServer or StorageEngine clusters register node information with the PCC.
    //You need to persist storage node information, update caches, and so on.
    pub fn cluster_register_node(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: RegisterNodeRequest = RegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_type = req.cluster_type();
        let cluster_name = req.cluster_name;
        let node = Node {
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
                nodes: vec![req.node_id],
                create_time: now_mills(),
            };
            self.cluster_cache.add_cluster(cluster_info.clone());
            cluster_storage.save(cluster_info);
            cluster_storage.save_all_cluster(cluster_name.clone());
        } else {
            cluster_storage.add_cluster_node(&cluster_name, node.node_id);
        }

        // update node
        self.cluster_cache.add_node(node.clone());

        node_storage.save_node(
            cluster_name.clone(),
            cluster_type.as_str_name().to_string(),
            node,
        );

        return Ok(());
    }

    // If a node is removed from the cluster,
    // the client may leave the cluster voluntarily or the node is removed because the heartbeat detection fails.
    pub fn cluster_unregister_node(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_name = req.cluster_name;
        let node_id = req.node_id;
        self.cluster_cache
            .remove_node(cluster_name.clone(), node_id);

        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        let cluster_storage = ClusterStorage::new(self.rocksdb_engine_handler.clone());
        node_storage.delete_node(&cluster_name, node_id);
        cluster_storage.remove_cluster_node(&cluster_name, node_id);
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        cache::{cluster::ClusterCache, journal::JournalCache},
        raft::route::cluster::DataRouteCluster,
        storage::{
            cluster::cluster::ClusterStorage, journal::shard::ShardStorage, cluster::node::NodeStorage, rocksdb::RocksDBEngine
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
        let node_port = 8763;

        let mut req = RegisterNodeRequest::default();
        req.node_id = node_id;
        req.node_ip = node_ip.clone();
        req.cluster_type = ClusterType::MqttBrokerServer.into();
        req.cluster_name = cluster_name.clone();
        req.extend_info = "{}".to_string();
        let data = RegisterNodeRequest::encode_to_vec(&req);

        let rocksdb_engine = Arc::new(RocksDBEngine::new(&PlacementCenterConfig::default()));

        let cluster_cache = Arc::new(ClusterCache::new());
        let engine_cache = Arc::new(JournalCache::new());

        let mut route = DataRouteCluster::new(rocksdb_engine.clone(), cluster_cache);
        let _ = route.cluster_register_node(data);

        let node_storage = NodeStorage::new(rocksdb_engine.clone());
        let cluster_storage = ClusterStorage::new(rocksdb_engine.clone());
        let shard_storage = ShardStorage::new(rocksdb_engine.clone());

        let cluster = cluster_storage.get(&cluster_name);
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
        assert_eq!(cl.nodes, vec![node_id]);

        let node = node_storage.get_node(cluster_name.clone(), node_id);
        let nd = node.unwrap();
        assert_eq!(nd.node_id, node_id);
        assert_eq!(nd.node_ip, node_ip);

        let _ = node_storage.delete_node(&cluster_name, node_id);
        let res = node_storage.get_node(cluster_name.clone(), node_id);
        assert!(res.is_none());

        let cluster = cluster_storage.get(&cluster_name);
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
        assert_eq!(cl.nodes.len(), 0);
    }
}
