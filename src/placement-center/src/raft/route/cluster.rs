// Copyright 2023 RobustMQ Team
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

use crate::{
    cache::placement::PlacementCacheManager,
    storage::{
        placement::{
            cluster::ClusterStorage, config::ResourceConfigStorage, idempotent::IdempotentStorage,
            node::NodeStorage,
        },
        rocksdb::RocksDBEngine,
    },
};
use common_base::{
    error::common::CommonError,
    tools::{now_mills, unique_id},
};
use metadata_struct::placement::{broker_node::BrokerNode, cluster::ClusterInfo};
use prost::Message as _;
use protocol::placement_center::generate::placement::{
    DeleteIdempotentDataRequest, DeleteResourceConfigRequest, RegisterNodeRequest,
    SetIdempotentDataRequest, SetResourceConfigRequest, UnRegisterNodeRequest,
};
use std::sync::Arc;

pub struct DataRouteCluster {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cluster_cache: Arc<PlacementCacheManager>,
}

impl DataRouteCluster {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cluster_cache: Arc<PlacementCacheManager>,
    ) -> Self {
        return DataRouteCluster {
            rocksdb_engine_handler,
            cluster_cache,
        };
    }

    pub fn add_node(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req: RegisterNodeRequest = RegisterNodeRequest::decode(value.as_ref())?;
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
            return cluster_storage.save(cluster_info);
        }

        // update node
        self.cluster_cache.add_node(node.clone());
        return node_storage.save(&cluster_name, node);
    }

    pub fn delete_node(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())?;
        let cluster_name = req.cluster_name;
        let node_id = req.node_id;
        self.cluster_cache.remove_node(&cluster_name, node_id);

        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        return node_storage.delete(&cluster_name, node_id);
    }

    pub fn set_resource_config(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = SetResourceConfigRequest::decode(value.as_ref())?;
        let config_storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        return config_storage.save(req.cluster_name, req.resources, req.config);
    }

    pub fn delete_resource_config(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = DeleteResourceConfigRequest::decode(value.as_ref())?;
        let config_storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        return config_storage.delete(req.cluster_name, req.resources);
    }

    pub fn set_idempotent_data(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = SetIdempotentDataRequest::decode(value.as_ref())?;
        let idempotent_storage = IdempotentStorage::new(self.rocksdb_engine_handler.clone());
        return idempotent_storage.save(&req.cluster_name, &req.producer_id, req.seq_num);
    }

    pub fn delete_idempotent_data(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = DeleteIdempotentDataRequest::decode(value.as_ref())?;
        let idempotent_storage = IdempotentStorage::new(self.rocksdb_engine_handler.clone());
        return idempotent_storage.delete(&req.cluster_name, &req.producer_id, req.seq_num);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        cache::placement::PlacementCacheManager,
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
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine.clone()));

        let route = DataRouteCluster::new(rocksdb_engine.clone(), cluster_cache);
        let _ = route.add_node(data);

        let node_storage = NodeStorage::new(rocksdb_engine.clone());
        let cluster_storage = ClusterStorage::new(rocksdb_engine.clone());

        let cluster = cluster_storage
            .get(
                &ClusterType::MqttBrokerServer.as_str_name().to_string(),
                &cluster_name,
            )
            .unwrap();
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);

        let node = node_storage.get(&cluster_name, node_id).unwrap();
        let nd = node.unwrap();
        assert_eq!(nd.node_id, node_id);
        assert_eq!(nd.node_ip, node_ip);

        let _ = node_storage.delete(&cluster_name, node_id);
        let res = node_storage.get(&cluster_name, node_id).unwrap();
        assert!(res.is_none());

        let cluster = cluster_storage
            .get(
                &ClusterType::MqttBrokerServer.as_str_name().to_string(),
                &cluster_name,
            )
            .unwrap();
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);
    }
}
