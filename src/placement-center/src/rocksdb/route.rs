use crate::{
    cache::{broker_cluster::BrokerClusterCache, engine_cluster::EngineClusterCache},
    rocksdb::{
        data_rw_layer::{ClusterInfo, NodeInfo, ShardInfo, ShardStatus},
        schema::{StorageData, StorageDataType},
    },
};
use bincode::deserialize;
use common::{errors::RobustMQError, tools::unique_id};
use prost::Message as _;
use protocol::placement_center::placement::{
    ClusterType, CreateShardRequest, RegisterNodeRequest, UnRegisterNodeRequest,
};
use std::sync::{Arc, RwLock};
use tonic::Status;

use super::data_rw_layer::DataRwLayer;

#[derive(Clone)]
pub struct DataRoute {
    cluster_storage: Arc<DataRwLayer>,
    engine_cache: Arc<RwLock<EngineClusterCache>>,
    broker_cache: Arc<RwLock<BrokerClusterCache>>,
}

impl DataRoute {
    pub fn new(
        cluster_storage: Arc<DataRwLayer>,
        engine_cache: Arc<RwLock<EngineClusterCache>>,
        broker_cache: Arc<RwLock<BrokerClusterCache>>,
    ) -> DataRoute {
        return DataRoute {
            cluster_storage,
            engine_cache,
            broker_cache,
        };
    }

    //Receive write operations performed by the Raft state machine and write subsequent service data after Raft state machine synchronization is complete.
    pub fn route(&self, data: Vec<u8>) -> Result<(), RobustMQError> {
        let storage_data: StorageData = deserialize(data.as_ref()).unwrap();
        match storage_data.data_type {
            StorageDataType::RegisterNode => {
                return self.register_node(storage_data.value);
            }
            StorageDataType::UngisterNode => {
                return self.unregister_node(storage_data.value);
            }
            StorageDataType::CreateShard => {
                return self.create_shard(storage_data.value);
            }
            StorageDataType::DeleteShard => {
                return self.create_shard(storage_data.value);
            }
        }
    }

    //BrokerServer or StorageEngine clusters register node information with the PCC.
    //You need to persist storage node information, update caches, and so on.
    pub fn register_node(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: RegisterNodeRequest = RegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_type = req.cluster_type();
        let cluster_name = req.cluster_name;

        let cluster_info = ClusterInfo {
            cluster_name: cluster_name.clone(),
            cluster_type: cluster_type.as_str_name().to_string(),
            nodes: Vec::new(),
        };

        let mut node = NodeInfo::default();
        node.node_id = req.node_id;
        node.node_ip = req.node_ip;
        node.node_port = req.node_port;

        if cluster_type == ClusterType::BrokerServer {
            // todo
        }

        if cluster_type == ClusterType::StorageEngine {
            let mut sc = self.engine_cache.write().unwrap();
            if !sc.cluster_list.contains_key(&cluster_name) {
                sc.add_cluster(cluster_info);
            }
            sc.add_node(node.clone());
            // todo
        }

        self.cluster_storage
            .save_node(cluster_name, cluster_type.as_str_name().to_string(), node);

        return Ok(());
    }

    // If a node is removed from the cluster, the client may leave the cluster voluntarily or the node is removed because the heartbeat detection fails.
    pub fn unregister_node(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_type = req.cluster_type();

        if cluster_type.eq(&ClusterType::BrokerServer) {
            // todo
        }

        if req.cluster_type().eq(&ClusterType::StorageEngine) {
            let mut sc = self.engine_cache.write().unwrap();
            sc.remove_node(req.node_id);
            // todo
        }

        self.cluster_storage
            .remove_node(req.cluster_name, req.node_id);

        return Ok(());
    }

    pub fn create_shard(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();

        // save shard info
        let mut shard_info = ShardInfo::default();
        shard_info.shard_id = unique_id();
        shard_info.shard_name = req.shard_name;
        shard_info.replica = req.replica;
        shard_info.replicas = Vec::new(); //todo Computing replica distribution
        shard_info.status = ShardStatus::Idle;
        self.cluster_storage
            .save_shard(req.cluster_name, shard_info.clone());

        let mut sc = self.engine_cache.write().unwrap();
        sc.add_shard(shard_info);
        // create next segment

        return Ok(());
    }

    pub fn delete_shard(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        // delete all segment

        // delete shard info
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        self.cluster_storage
            .delete_shard(req.cluster_name, req.shard_name);
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{cache::{broker_cluster::BrokerClusterCache, engine_cluster::EngineClusterCache}, rocksdb::{data_rw_layer::DataRwLayer, rocksdb::RocksDBStorage}};
    use common::config::placement_center::PlacementCenterConfig;
    use prost::Message as _;
    use protocol::placement_center::placement::{ClusterType, RegisterNodeRequest};

    use super::DataRoute;

    #[test]
    fn register_unregister_node() {
        let cluster_name = "test-cluster".to_string();
        let node_id = 1;
        let node_ip = "127.0.0.1".to_string();
        let node_port = 8763;

        let mut req = RegisterNodeRequest::default();
        req.node_id = node_id;
        req.node_ip = node_ip.clone();
        req.node_port = node_port;
        req.cluster_type = ClusterType::BrokerServer.into();
        req.cluster_name = cluster_name.clone();
        req.extend_info = "{}".to_string();
        let data = RegisterNodeRequest::encode_to_vec(&req);

        let rocksdb_storage = Arc::new(RocksDBStorage::new(&PlacementCenterConfig::default()));
        let cluster_storage = Arc::new(DataRwLayer::new(rocksdb_storage));
        let broker_cache = Arc::new(RwLock::new(BrokerClusterCache::new()));
        let engine_cache = Arc::new(RwLock::new(EngineClusterCache::new()));
        let mut route = DataRoute::new(cluster_storage.clone(), engine_cache, broker_cache);
        let _ = route.register_node(data);

        let cluster = cluster_storage.get_cluster(&cluster_name);
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
        assert_eq!(cl.nodes, vec![node_id]);

        let node = cluster_storage.get_node(cluster_name.clone(), node_id);
        let nd = node.unwrap();
        assert_eq!(nd.node_id, node_id);
        assert_eq!(nd.node_ip, node_ip);
        assert_eq!(nd.node_port, node_port);

        let _ = cluster_storage.remove_node(cluster_name.clone(), node_id);
        let res = cluster_storage.get_node(cluster_name.clone(), node_id);
        assert!(res.is_none());

        let cluster = cluster_storage.get_cluster(&cluster_name);
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
        assert_eq!(cl.nodes.len(), 0);
    }
}
