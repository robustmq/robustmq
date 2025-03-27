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

use metadata_struct::placement::cluster::ClusterInfo;
use metadata_struct::placement::node::BrokerNode;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use prost::Message as _;
use protocol::placement_center::placement_center_inner::{
    BindSchemaRequest, CreateSchemaRequest, DeleteIdempotentDataRequest,
    DeleteResourceConfigRequest, DeleteSchemaRequest, SaveOffsetDataRequest,
    SetIdempotentDataRequest, SetResourceConfigRequest, UnBindSchemaRequest, UnRegisterNodeRequest,
};
use std::sync::Arc;

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::storage::placement::cluster::ClusterStorage;
use crate::storage::placement::config::ResourceConfigStorage;
use crate::storage::placement::idempotent::IdempotentStorage;
use crate::storage::placement::node::NodeStorage;
use crate::storage::placement::offset::OffsetStorage;
use crate::storage::placement::schema::SchemaStorage;
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

    // Cluster
    pub async fn add_cluster(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let cluster = serde_json::from_slice::<ClusterInfo>(&value)?;
        let cluster_storage = ClusterStorage::new(self.rocksdb_engine_handler.clone());
        cluster_storage.save(&cluster)?;
        self.cluster_cache.add_broker_cluster(&cluster);
        Ok(())
    }

    // Node
    pub async fn add_node(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let node = serde_json::from_slice::<BrokerNode>(&value)?;
        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        node_storage.save(&node)?;
        self.cluster_cache.add_broker_node(node);
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

    // ResourceConfig
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

    // IdempotentData
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

    // OffsetData
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

    // Schema
    pub fn set_schema(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateSchemaRequest::decode(value.as_ref())?;
        let schema_storage = SchemaStorage::new(self.rocksdb_engine_handler.clone());
        let schema = serde_json::from_slice::<SchemaData>(&req.schema)?;
        schema_storage.save(&req.cluster_name, &req.schema_name, &schema)?;
        Ok(())
    }

    pub fn delete_schema(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteSchemaRequest::decode(value.as_ref())?;
        let schema_storage = SchemaStorage::new(self.rocksdb_engine_handler.clone());
        schema_storage.delete(&req.cluster_name, &req.schema_name)?;
        Ok(())
    }

    // Schema Bind
    pub fn set_schema_bind(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = BindSchemaRequest::decode(value.as_ref())?;
        let schema_storage = SchemaStorage::new(self.rocksdb_engine_handler.clone());
        let bind_data = SchemaResourceBind {
            cluster_name: req.cluster_name.clone(),
            resource_name: req.resource_name.clone(),
            schema_name: req.schema_name.clone(),
        };
        schema_storage.save_bind(&req.cluster_name, bind_data)?;
        Ok(())
    }

    pub fn delete_schema_bind(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = UnBindSchemaRequest::decode(value.as_ref())?;
        let schema_storage = SchemaStorage::new(self.rocksdb_engine_handler.clone());
        schema_storage.delete_bind(&req.cluster_name, &req.resource_name, &req.schema_name)?;
        Ok(())
    }

    pub fn delete_offset_data(&self, _: Vec<u8>) -> Result<(), PlacementCenterError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::tools::unique_id;
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::placement::node::BrokerNode;
    use protocol::placement_center::placement_center_inner::ClusterType;

    use crate::core::cache::PlacementCacheManager;
    use crate::route::common::DataRouteCluster;
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
            &test_temp_dir(),
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
    }
}
