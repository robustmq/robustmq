use crate::{
    broker_cluster::BrokerCluster,
    errors::MetaError,
    storage::{
        cluster_storage::{NodeInfo, ShardInfo, ShardStatus},
        schema::{StorageData, StorageDataType},
    },
    storage_cluster::StorageCluster,
};
use bincode::deserialize;
use common::tools::unique_id;
use prost::Message as _;
use protocol::placement_center::placement::{
    CreateShardRequest, RegisterNodeRequest, UnRegisterNodeRequest,
};
use std::sync::Arc;
use tonic::Status;

use super::cluster_storage::ClusterStorage;

#[derive(Clone)]
pub struct DataRoute {
    cluster_storage: Arc<ClusterStorage>,
    storage_cluster: Arc<StorageCluster>,
    broker_cluster: Arc<BrokerCluster>,
}

impl DataRoute {
    pub fn new(
        cluster_storage: Arc<ClusterStorage>,
        storage_cluster: Arc<StorageCluster>,
        broker_cluster: Arc<BrokerCluster>,
    ) -> DataRoute {
        return DataRoute {
            cluster_storage,
            storage_cluster,
            broker_cluster,
        };
    }

    pub fn route(&self, data: Vec<u8>) -> Result<(), MetaError> {
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

    pub fn register_node(&self, value: Vec<u8>) -> Result<(), MetaError> {
        let req: RegisterNodeRequest = RegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();

        let cluster_name = req.cluster_name;
        let mut node = NodeInfo::default();
        node.node_id = req.node_id;
        node.node_ip = req.node_ip;
        node.node_port = req.node_port;
        self.cluster_storage
            .save_node(cluster_name, req.node_type.to_string(), node);
        return Ok(());
    }

    pub fn unregister_node(&self, value: Vec<u8>) -> Result<(), MetaError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();

        self.cluster_storage
            .remove_node(req.cluster_name, req.node_id);
        return Ok(());
    }

    pub fn create_shard(&self, value: Vec<u8>) -> Result<(), MetaError> {
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();

        // save shard info
        let mut shard_info = ShardInfo::default();
        shard_info.shard_id = unique_id();
        shard_info.shard_name = req.shard_name;
        shard_info.replica = req.replica;

        //todo Computing replica distribution
        shard_info.replicas = Vec::new();
        shard_info.status = ShardStatus::Idle;
        self.cluster_storage
            .save_shard(req.cluster_name, shard_info);

        // create next segment

        return Ok(());
    }

    pub fn delete_shard(&self, value: Vec<u8>) -> Result<(), MetaError> {
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
    use std::sync::Arc;

    use crate::{
        broker_cluster::BrokerCluster,
        storage::{cluster_storage::ClusterStorage, rocksdb::RocksDBStorage},
        storage_cluster::StorageCluster,
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
        req.node_type = ClusterType::BrokerServer.into();
        req.cluster_name = cluster_name.clone();
        req.extend_info = "{}".to_string();
        let data = RegisterNodeRequest::encode_to_vec(&req);

        let rocksdb_storage = Arc::new(RocksDBStorage::new(&PlacementCenterConfig::default()));
        let cluster_storage = Arc::new(ClusterStorage::new(rocksdb_storage));
        let broker_cluster = Arc::new(BrokerCluster::new());
        let storage_cluster = Arc::new(StorageCluster::new());
        let route = DataRoute::new(cluster_storage.clone(), storage_cluster, broker_cluster);
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
