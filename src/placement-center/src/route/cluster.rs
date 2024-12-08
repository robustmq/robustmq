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

use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::placement::cluster::ClusterInfo;
use metadata_struct::placement::node::BrokerNode;
use prost::Message as _;
use protocol::placement_center::placement_center_inner::{
    DeleteIdempotentDataRequest, DeleteResourceConfigRequest, SaveOffsetDataRequest,
    SetIdempotentDataRequest, SetResourceConfigRequest, UnRegisterNodeRequest,
};
use protocol::placement_center::placement_center_mqtt::{
    CreateAclRequest, CreateBlacklistRequest, DeleteAclRequest, DeleteBlacklistRequest,
};

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::storage::mqtt::acl::AclStorage;
use crate::storage::mqtt::blacklist::MqttBlackListStorage;
use crate::storage::placement::cluster::ClusterStorage;
use crate::storage::placement::config::ResourceConfigStorage;
use crate::storage::placement::idempotent::IdempotentStorage;
use crate::storage::placement::node::NodeStorage;
use crate::storage::placement::offset::OffsetStorage;
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Clone)]
pub struct DataRouteCluster {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cluster_cache: Arc<PlacementCacheManager>,
}

impl DataRouteCluster {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cluster_cache: Arc<PlacementCacheManager>,
    ) -> Self {
        DataRouteCluster {
            rocksdb_engine_handler,
            cluster_cache,
        }
    }

    pub async fn add_node(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let node = serde_json::from_slice::<BrokerNode>(&value)?;

        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        node_storage.save(&node)?;

        self.cluster_cache.add_broker_node(node);

        Ok(())
    }

    pub async fn add_cluster(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let cluster = serde_json::from_slice::<ClusterInfo>(&value)?;

        let cluster_storage = ClusterStorage::new(self.rocksdb_engine_handler.clone());
        cluster_storage.save(&cluster)?;

        self.cluster_cache.add_broker_cluster(&cluster);
        Ok(())
    }

    pub async fn delete_node(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())?;
        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        node_storage.delete(&req.cluster_name, req.node_id)?;
        self.cluster_cache
            .remove_broker_node(&req.cluster_name, req.node_id);
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

    pub fn save_offset_data(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = SaveOffsetDataRequest::decode(value.as_ref())?;
        let offset_storage = OffsetStorage::new(self.rocksdb_engine_handler.clone());
        for raw in req.offsets {
            offset_storage.save(
                &req.cluster_name,
                &req.group,
                &raw.namespace,
                &raw.shard_name,
                raw.offset,
            )?;
        }
        Ok(())
    }

    pub fn delete_offset_data(&self, _: Vec<u8>) -> Result<(), PlacementCenterError> {
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
}

#[cfg(test)]
mod tests {
    use std::fs::remove_dir_all;
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::tools::unique_id;
    use metadata_struct::placement::node::BrokerNode;
    use protocol::placement_center::placement_center_inner::ClusterType;

    use crate::core::cache::PlacementCacheManager;
    use crate::route::cluster::DataRouteCluster;
    use crate::storage::placement::node::NodeStorage;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[tokio::test]
    async fn register_unregister_node() {
        let config = placement_center_test_conf();

        let cluster_name = unique_id();
        let node_id = 999;
        let node_ip = "127.0.0.1".to_string();

        let node = BrokerNode {
            cluster_name: cluster_name.clone(),
            node_id,
            node_ip: node_ip.clone(),
            cluster_type: ClusterType::MqttBrokerServer.as_str_name().to_string(),
            ..Default::default()
        };
        let data = serde_json::to_vec(&node).unwrap();
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine.clone()));
        let route = DataRouteCluster::new(rocksdb_engine.clone(), cluster_cache);
        route.add_node(data).await.unwrap();

        let node_storage = NodeStorage::new(rocksdb_engine.clone());
        let node = node_storage.get(&cluster_name, node_id).unwrap();
        let broker_node = node.unwrap();
        assert_eq!(broker_node.node_id, node_id);
        assert_eq!(broker_node.node_ip, node_ip);

        let _ = node_storage.delete(&cluster_name, node_id);
        let res = node_storage.get(&cluster_name, node_id).unwrap();
        assert!(res.is_none());

        remove_dir_all(config.rocksdb.data_path).unwrap();
    }
}
