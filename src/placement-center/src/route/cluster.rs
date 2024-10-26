// Copyright 2023 RobustMQ Team
//
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

use std::sync::Arc;

use common_base::tools::{now_mills, unique_id};
use grpc_clients::poll::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::placement::cluster::ClusterInfo;
use metadata_struct::placement::node::BrokerNode;
use prost::Message as _;
use protocol::placement_center::placement_center_inner::{
    ClusterType, DeleteIdempotentDataRequest, DeleteResourceConfigRequest, RegisterNodeRequest,
    SetIdempotentDataRequest, SetResourceConfigRequest, UnRegisterNodeRequest,
};
use protocol::placement_center::placement_center_mqtt::{
    CreateAclRequest, CreateBlacklistRequest, DeleteAclRequest, DeleteBlacklistRequest,
};

use crate::cache::placement::PlacementCacheManager;
use crate::controller::journal::call_node::{
    update_cache_by_add_journal_node, update_cache_by_delete_journal_node, JournalInnerCallManager,
};
use crate::core::error::PlacementCenterError;
use crate::storage::mqtt::acl::AclStorage;
use crate::storage::mqtt::blacklist::MqttBlackListStorage;
use crate::storage::placement::cluster::ClusterStorage;
use crate::storage::placement::config::ResourceConfigStorage;
use crate::storage::placement::idempotent::IdempotentStorage;
use crate::storage::placement::node::NodeStorage;
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Clone)]
pub struct DataRouteCluster {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cluster_cache: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
    call_manager: Arc<JournalInnerCallManager>,
}

impl DataRouteCluster {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cluster_cache: Arc<PlacementCacheManager>,
        call_manager: Arc<JournalInnerCallManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        DataRouteCluster {
            rocksdb_engine_handler,
            cluster_cache,
            call_manager,
            client_poll,
        }
    }

    pub async fn register_node(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
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
            self.cluster_cache.add_broker_cluster(&cluster_info);
            cluster_storage.save(&cluster_info)?;
        }

        // update node
        self.cluster_cache.add_broker_node(node.clone());
        node_storage.save(&node)?;

        // Call Broker/Journal to refresh the cluster cache
        self.call_add_node_cache(&node).await?;

        Ok(())
    }

    pub async fn unregister_node(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())?;
        let cluster_name = req.cluster_name;
        let node_id = req.node_id;

        if let Some((_, node)) = self
            .cluster_cache
            .remove_broker_node(&cluster_name, node_id)
        {
            let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
            node_storage.delete(&cluster_name, node_id)?;

            // Call Broker/Journal to refresh the cluster cache
            self.call_delete_node_cache(&node).await?;
        }

        Ok(())
    }

    pub fn set_resource_config(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = SetResourceConfigRequest::decode(value.as_ref())?;
        let config_storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        config_storage.save(req.cluster_name, req.resources, req.config)?;
        Ok(())
    }

    pub fn delete_resource_config(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteResourceConfigRequest::decode(value.as_ref())?;
        let config_storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        config_storage.delete(req.cluster_name, req.resources)?;
        Ok(())
    }

    pub fn set_idempotent_data(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = SetIdempotentDataRequest::decode(value.as_ref())?;
        let idempotent_storage = IdempotentStorage::new(self.rocksdb_engine_handler.clone());
        idempotent_storage.save(&req.cluster_name, &req.producer_id, req.seq_num)?;
        Ok(())
    }

    pub fn delete_idempotent_data(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteIdempotentDataRequest::decode(value.as_ref())?;
        let idempotent_storage = IdempotentStorage::new(self.rocksdb_engine_handler.clone());
        idempotent_storage.delete(&req.cluster_name, &req.producer_id, req.seq_num)?;
        Ok(())
    }

    pub fn create_acl(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateAclRequest::decode(value.as_ref())?;
        let acl_storage = AclStorage::new(self.rocksdb_engine_handler.clone());
        let acl = serde_json::from_slice::<MqttAcl>(&req.acl)?;
        acl_storage.save(&req.cluster_name, acl)?;
        Ok(())
    }

    pub fn delete_acl(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteAclRequest::decode(value.as_ref())?;
        let acl_storage = AclStorage::new(self.rocksdb_engine_handler.clone());
        let acl = serde_json::from_slice::<MqttAcl>(&req.acl)?;
        acl_storage.delete(&req.cluster_name, &acl)?;
        Ok(())
    }

    pub fn create_blacklist(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateBlacklistRequest::decode(value.as_ref())?;
        let blacklist_storage = MqttBlackListStorage::new(self.rocksdb_engine_handler.clone());
        let blacklist = serde_json::from_slice::<MqttAclBlackList>(&req.blacklist)?;
        blacklist_storage.save(&req.cluster_name, blacklist)?;
        Ok(())
    }

    pub fn delete_blacklist(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteBlacklistRequest::decode(value.as_ref())?;
        let blacklist_storage = MqttBlackListStorage::new(self.rocksdb_engine_handler.clone());
        blacklist_storage.delete(&req.cluster_name, &req.blacklist_type, &req.resource_name)?;
        Ok(())
    }

    async fn call_add_node_cache(&self, node: &BrokerNode) -> Result<(), PlacementCenterError> {
        if node.cluster_type == *ClusterType::JournalServer.as_str_name() {
            update_cache_by_add_journal_node(
                &node.cluster_name,
                &self.call_manager,
                &self.client_poll,
                node.clone(),
            )
            .await?;
        }
        Ok(())
    }

    async fn call_delete_node_cache(&self, node: &BrokerNode) -> Result<(), PlacementCenterError> {
        if node.cluster_type == *ClusterType::JournalServer.as_str_name() {
            update_cache_by_delete_journal_node(
                &node.cluster_name,
                &self.call_manager,
                &self.client_poll,
                node.clone(),
            )
            .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_dir_all;
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::tools::unique_id;
    use grpc_clients::poll::ClientPool;
    use prost::Message as _;
    use protocol::placement_center::placement_center_inner::{ClusterType, RegisterNodeRequest};

    use crate::cache::placement::PlacementCacheManager;
    use crate::controller::journal::call_node::JournalInnerCallManager;
    use crate::route::cluster::DataRouteCluster;
    use crate::storage::placement::cluster::ClusterStorage;
    use crate::storage::placement::node::NodeStorage;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[tokio::test]
    async fn register_unregister_node() {
        let config = placement_center_test_conf();

        let cluster_name = unique_id();
        let node_id = 999;
        let node_ip = "127.0.0.1".to_string();

        let req = RegisterNodeRequest {
            node_id,
            node_ip: node_ip.clone(),
            cluster_type: ClusterType::MqttBrokerServer.into(),
            cluster_name: cluster_name.clone(),
            extend_info: "{}".to_string(),
            ..Default::default()
        };
        let data = RegisterNodeRequest::encode_to_vec(&req);
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine.clone()));
        let client_poll = Arc::new(ClientPool::new(3));
        let call_manager = Arc::new(JournalInnerCallManager::new(cluster_cache.clone()));
        let route = DataRouteCluster::new(
            rocksdb_engine.clone(),
            cluster_cache,
            call_manager,
            client_poll,
        );
        route.register_node(data).await.unwrap();

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
            .get(ClusterType::MqttBrokerServer.as_str_name(), &cluster_name)
            .unwrap();
        let cl = cluster.unwrap();
        assert_eq!(cl.cluster_name, cluster_name);

        remove_dir_all(config.rocksdb.data_path).unwrap();
    }
}
