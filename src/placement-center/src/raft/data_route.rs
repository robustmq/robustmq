use super::storage::{StorageData, StorageDataType};
use crate::{
    cache::{broker::BrokerClusterCache, engine::EngineClusterCache},
    controller::engine::segment_replica::SegmentReplicaAlgorithm,
    storage::{
        cluster::{ClusterInfo, ClusterStorage},
        kv::KvStorage,
        node::{NodeInfo, NodeStorage},
        rocksdb::RocksDBEngine,
        segment::{SegmentInfo, SegmentStatus, SegmentStorage},
        shard::{ShardInfo, ShardStorage},
    },
};
use bincode::deserialize;
use common_base::{
    errors::RobustMQError,
    tools::{now_mills, unique_id},
};
use prost::Message as _;
use protocol::placement_center::generate::{common::ClusterType, engine::{CreateSegmentRequest, CreateShardRequest, DeleteSegmentRequest}, kv::{DeleteRequest, SetRequest}, placement::{RegisterNodeRequest, UnRegisterNodeRequest}};
use std::sync::{Arc, RwLock};
use tonic::Status;

pub struct DataRoute {
    engine_cache: Arc<RwLock<EngineClusterCache>>,
    broker_cache: Arc<RwLock<BrokerClusterCache>>,
    node_storage: NodeStorage,
    cluster_storage: ClusterStorage,
    shard_storage: ShardStorage,
    segment_storage: SegmentStorage,
    kv_storage: KvStorage,
    repcli_algo: SegmentReplicaAlgorithm,
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
        let segment_storage = SegmentStorage::new(rocksdb_engine_handler.clone());
        let kv_storage = KvStorage::new(rocksdb_engine_handler);
        let repcli_algo = SegmentReplicaAlgorithm::new(engine_cache.clone());
        return DataRoute {
            engine_cache,
            broker_cache,
            node_storage,
            cluster_storage,
            shard_storage,
            segment_storage,
            kv_storage,
            repcli_algo,
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
                return self.delete_shard(storage_data.value);
            }

            StorageDataType::CreateSegment => {
                return self.create_segment(storage_data.value);
            }
            StorageDataType::DeleteSegment => {
                return self.delete_segment(storage_data.value);
            }
            StorageDataType::Set => {
                return self.set(storage_data.value);
            }
            StorageDataType::Delete => {
                return self.delete(storage_data.value);
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

        let node = NodeInfo {
            node_id: req.node_id,
            node_ip: req.node_ip,
            node_port: req.node_port,
            cluster_name: cluster_name.clone(),
            create_time: now_mills(),
        };

        if cluster_type == ClusterType::MqttBrokerServer {
            // todo
        }

        // Update the information in the StorageEngine cache
        if cluster_type == ClusterType::StorageEngine {
            // update cluster
            let mut sc = self.engine_cache.write().unwrap();
            if !sc.cluster_list.contains_key(&cluster_name) {
                let cluster_info = ClusterInfo {
                    cluster_uid: unique_id(),
                    cluster_name: cluster_name.clone(),
                    cluster_type: cluster_type.as_str_name().to_string(),
                    nodes: vec![req.node_id],
                    create_time: now_mills(),
                };
                sc.add_cluster(cluster_info.clone());
                self.cluster_storage.save_cluster(cluster_info);
                self.cluster_storage.save_all_cluster(cluster_name.clone());
            } else {
                sc.add_cluster_node(cluster_name.clone(), node.node_id);
                self.cluster_storage
                    .add_cluster_node(&cluster_name, node.node_id);
            }

            // update node
            sc.add_node(node.clone());
        }

        self.node_storage.save_node(
            cluster_name.clone(),
            cluster_type.as_str_name().to_string(),
            node,
        );

        return Ok(());
    }

    // If a node is removed from the cluster,
    // the client may leave the cluster voluntarily or the node is removed because the heartbeat detection fails.
    pub fn unregister_node(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_type = req.cluster_type();
        let cluster_name = req.cluster_name;
        let node_id = req.node_id;

        if cluster_type.eq(&ClusterType::MqttBrokerServer) {
            // todo
        }

        if cluster_type.eq(&ClusterType::StorageEngine) {
            let mut sc = self.engine_cache.write().unwrap();
            sc.remove_cluster_node(cluster_name.clone(), node_id);
            sc.remove_node(cluster_name.clone(), node_id);
        }

        self.node_storage.delete_node(&cluster_name, node_id);
        self.cluster_storage
            .remove_cluster_node(&cluster_name, node_id);
        return Ok(());
    }

    pub fn create_shard(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();

        let cluster_name = req.cluster_name;

        let shard_info = ShardInfo {
            shard_uid: unique_id(),
            cluster_name: cluster_name.clone(),
            shard_name: req.shard_name.clone(),
            replica: req.replica,
            last_segment_seq: 0,
            segments: Vec::new(),
            create_time: now_mills(),
        };

        // persist
        self.shard_storage.save_shard(shard_info.clone());
        self.shard_storage
            .save_all_shard(cluster_name.clone(), shard_info.shard_name.clone());

        // upate cache
        let mut sc = self.engine_cache.write().unwrap();
        sc.add_shard(shard_info);

        // Create segments according to the built-in pre-created segment strategy.
        // Between 0 and N segments may be created.
        self.pre_create_segment().unwrap();

        // todo maybe update storage engine node cache
        return Ok(());
    }

    pub fn delete_shard(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_name = req.cluster_name.clone();
        let shard_name = req.shard_name.clone();
        // todo delete all segment

        // delete shard info
        self.shard_storage
            .delete_shard(cluster_name.clone(), shard_name.clone());
        let mut sc = self.engine_cache.write().unwrap();
        sc.remove_shard(cluster_name.clone(), shard_name.clone());
        return Ok(());
    }

    pub fn create_segment(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: CreateSegmentRequest = CreateSegmentRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();

        let cluster_name = req.cluster_name;
        let shard_name = req.shard_name;

        let ec = self.engine_cache.read().unwrap();
        let segment_seq = ec.next_segment_seq(&cluster_name, &shard_name);
        drop(ec);

        let segment_info = SegmentInfo {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            replicas: self.repcli_algo.calc_replica_distribution(segment_seq),
            replica_leader: 0,
            segment_seq: segment_seq,
            status: SegmentStatus::Idle,
        };

        // Updating cache information
        let mut ec = self.engine_cache.write().unwrap();
        ec.add_segment(segment_info.clone());

        // Update persistence information
        self.segment_storage.save_segment(segment_info);
        self.shard_storage
            .add_segment(cluster_name, shard_name, segment_seq);

        // todo call storage engine create segment
        return Ok(());
    }

    pub fn delete_segment(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: DeleteSegmentRequest = DeleteSegmentRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let cluster_name = req.cluster_name;
        let shard_name = req.shard_name;
        let segment_seq = req.segment_seq;

        // Updating cache information
        let mut ec = self.engine_cache.write().unwrap();
        ec.remove_segment(cluster_name.clone(), shard_name.clone(), segment_seq);

        // Update persistence information
        self.segment_storage
            .delete_segment(cluster_name.clone(), shard_name.clone(), segment_seq);
        self.shard_storage
            .delete_segment(cluster_name.clone(), shard_name.clone(), segment_seq);

        // todo call storage engine delete segment
        return Ok(());
    }

    pub fn set(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: SetRequest = SetRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        self.kv_storage.set(req.key, req.value);
        return Ok(());
    }

    pub fn delete(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: DeleteRequest = DeleteRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        self.kv_storage.delete(req.key);
        return Ok(());
    }

    pub fn pre_create_segment(&self) -> Result<(), RobustMQError> {
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{
        cache::{broker::BrokerClusterCache, engine::EngineClusterCache},
        storage::{
            cluster::ClusterStorage, node::NodeStorage, rocksdb::RocksDBEngine, shard::ShardStorage,
        },
    };
    use common_base::config::placement_center::PlacementCenterConfig;
    use prost::Message as _;
    use protocol::placement_center::generate::{common::ClusterType, placement::RegisterNodeRequest};
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
        req.cluster_type = ClusterType::MqttBrokerServer.into();
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

        let _ = node_storage.delete_node(&cluster_name, node_id);
        let res = node_storage.get_node(cluster_name.clone(), node_id);
        assert!(res.is_none());

        let cluster = cluster_storage.get_cluster(&cluster_name);
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
        assert_eq!(cl.nodes.len(), 0);
    }
}
