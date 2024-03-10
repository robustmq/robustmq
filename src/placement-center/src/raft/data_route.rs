use crate::{
    cache::{broker_cluster::BrokerClusterCache, engine_cluster::EngineClusterCache},
    rocksdb::{
        cluster::{ClusterInfo, ClusterStorage},
        node::{NodeInfo, NodeStorage},
        rocksdb::RocksDBEngine,
        shard::{ShardInfo, ShardStatus, ShardStorage},
    },
};
use bincode::deserialize;
use common::{
    errors::RobustMQError,
    tools::{now_mills, unique_id},
};
use prost::Message as _;
use protocol::placement_center::placement::{
    ClusterType, CreateShardRequest, RegisterNodeRequest, UnRegisterNodeRequest,
};
use std::sync::{Arc, RwLock};
use tonic::Status;

use super::storage::{StorageData, StorageDataType};

pub struct DataRoute {
    engine_cache: Arc<RwLock<EngineClusterCache>>,
    broker_cache: Arc<RwLock<BrokerClusterCache>>,
    node_storage: NodeStorage,
    cluster_storage: ClusterStorage,
    shard_storage: ShardStorage,
}

impl DataRoute {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        engine_cache: Arc<RwLock<EngineClusterCache>>,
        broker_cache: Arc<RwLock<BrokerClusterCache>>,
    ) -> DataRoute {
        let node_storage = NodeStorage::new(rocksdb_engine_handler.clone());
        let cluster_storage = ClusterStorage::new(rocksdb_engine_handler.clone());
        let shard_storage = ShardStorage::new(rocksdb_engine_handler.clone());
        return DataRoute {
            engine_cache,
            broker_cache,
            node_storage,
            cluster_storage,
            shard_storage,
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

        let mut node = NodeInfo {
            node_uid: unique_id(),
            node_id: req.node_id,
            node_ip: req.node_ip,
            node_port: req.node_port,
            cluster_name: cluster_name.clone(),
            create_time: now_mills(),
        };

        if cluster_type == ClusterType::BrokerServer {
            // todo
        }

        // Update the information in the StorageEngine cache
        if cluster_type == ClusterType::StorageEngine {
            // update cluster cache
            let mut sc = self.engine_cache.write().unwrap();
            if !sc.cluster_list.contains_key(&cluster_name) {
                let cluster_info = ClusterInfo {
                    cluster_uid: unique_id(),
                    cluster_name: cluster_name.clone(),
                    cluster_type: cluster_type.as_str_name().to_string(),
                    nodes: vec![req.node_id],
                    create_time: now_mills(),
                };
                sc.add_cluster(cluster_info);
            } else {
                sc.add_cluster_node(cluster_name.clone(), node.node_id);
            }

            // update node cache
            sc.add_node(node.clone());
        }

        // Persisting storage node data
        self.node_storage
            .save_node(cluster_name, cluster_type.as_str_name().to_string(), node);

        return Ok(());
    }

    // If a node is removed from the cluster, the client may leave the cluster voluntarily or the node is removed because the heartbeat detection fails.
    pub fn unregister_node(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_type = req.cluster_type();
        let cluster_name = req.cluster_name;
        let node_id = req.node_id;

        if cluster_type.eq(&ClusterType::BrokerServer) {
            // todo
        }

        if cluster_type.eq(&ClusterType::StorageEngine) {
            let mut sc = self.engine_cache.write().unwrap();
            sc.remove_cluster_node(cluster_name.clone(), node_id);
            sc.remove_node(cluster_name.clone(), req.node_id);
        }

        self.node_storage.remove_node(cluster_name, req.node_id);

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
        self.shard_storage
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
        self.shard_storage
            .delete_shard(req.cluster_name, req.shard_name);
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{
        cache::{broker_cluster::BrokerClusterCache, engine_cluster::EngineClusterCache},
        rocksdb::{
            cluster::ClusterStorage, node::NodeStorage, rocksdb::RocksDBEngine, shard::ShardStorage,
        },
    };
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

        let rocksdb_engine = Arc::new(RocksDBEngine::new(&PlacementCenterConfig::default()));

        let broker_cache = Arc::new(RwLock::new(BrokerClusterCache::new()));
        let engine_cache = Arc::new(RwLock::new(EngineClusterCache::new()));

        let mut route = DataRoute::new(rocksdb_engine.clone(), engine_cache, broker_cache);
        let _ = route.register_node(data);

        let node_storage = NodeStorage::new(rocksdb_engine.clone());
        let cluster_storage = ClusterStorage::new(rocksdb_engine.clone());
        let shard_storage = ShardStorage::new(rocksdb_engine.clone());

        let cluster = cluster_storage.get_cluster(&cluster_name);
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
        assert_eq!(cl.nodes, vec![node_id]);

        let node = node_storage.get_node(cluster_name.clone(), node_id);
        let nd = node.unwrap();
        assert_eq!(nd.node_id, node_id);
        assert_eq!(nd.node_ip, node_ip);
        assert_eq!(nd.node_port, node_port);

        let _ = node_storage.remove_node(cluster_name.clone(), node_id);
        let res = node_storage.get_node(cluster_name.clone(), node_id);
        assert!(res.is_none());

        let cluster = cluster_storage.get_cluster(&cluster_name);
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
        assert_eq!(cl.nodes.len(), 0);
    }
}
