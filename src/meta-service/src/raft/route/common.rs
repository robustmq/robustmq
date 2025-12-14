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

use bytes::Bytes;
use common_base::tools::now_second;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use prost::Message as _;
use protocol::meta::meta_service_common::{
    BindSchemaRequest, CreateSchemaRequest, DeleteResourceConfigRequest, DeleteSchemaRequest,
    RegisterNodeRequest, SaveOffsetDataRequest, SetResourceConfigRequest, UnBindSchemaRequest,
    UnRegisterNodeRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::storage::common::config::ResourceConfigStorage;
use crate::storage::common::node::NodeStorage;
use crate::storage::common::offset::{OffsetData, OffsetStorage};
use crate::storage::common::schema::SchemaStorage;

#[derive(Clone)]
pub struct DataRouteCluster {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cluster_cache: Arc<CacheManager>,
}

impl DataRouteCluster {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cluster_cache: Arc<CacheManager>,
    ) -> Self {
        DataRouteCluster {
            rocksdb_engine_handler,
            cluster_cache,
        }
    }

    // Node
    pub async fn add_node(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = RegisterNodeRequest::decode(value.as_ref())?;
        let node = BrokerNode::decode(&req.node)?;
        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        node_storage.save(&node)?;
        self.cluster_cache.add_broker_node(node);
        Ok(())
    }

    pub async fn delete_node(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req: UnRegisterNodeRequest = UnRegisterNodeRequest::decode(value.as_ref())?;
        let node_storage = NodeStorage::new(self.rocksdb_engine_handler.clone());
        node_storage.delete(req.node_id)?;
        self.cluster_cache.remove_broker_node(req.node_id);
        Ok(())
    }

    // ResourceConfig
    pub fn set_resource_config(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetResourceConfigRequest::decode(value.as_ref())?;
        let config_storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        config_storage.save(req.resources, req.config)?;
        Ok(())
    }

    pub fn delete_resource_config(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteResourceConfigRequest::decode(value.as_ref())?;
        let config_storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        config_storage.delete(req.resources)?;
        Ok(())
    }

    // OffsetData
    pub fn save_offset_data(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SaveOffsetDataRequest::decode(value.as_ref())?;
        let offset_storage = OffsetStorage::new(self.rocksdb_engine_handler.clone());
        for offset_data in req.offsets {
            let offsets = offset_data
                .offsets
                .iter()
                .map(|raw| OffsetData {
                    namespace: raw.namespace.clone(),
                    group: offset_data.group.clone(),
                    shard_name: raw.shard_name.clone(),
                    offset: raw.offset,
                    timestamp: now_second(),
                })
                .collect::<Vec<OffsetData>>();

            offset_storage.save(&offsets)?;
        }
        Ok(())
    }

    // Schema
    pub fn set_schema(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateSchemaRequest::decode(value.as_ref())?;
        let schema_storage = SchemaStorage::new(self.rocksdb_engine_handler.clone());
        let schema = SchemaData::decode(&req.schema)?;
        schema_storage.save(&req.schema_name, &schema)?;
        Ok(())
    }

    pub fn delete_schema(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteSchemaRequest::decode(value.as_ref())?;
        let schema_storage = SchemaStorage::new(self.rocksdb_engine_handler.clone());
        schema_storage.delete(&req.schema_name)?;
        Ok(())
    }

    // Schema Bind
    pub fn set_schema_bind(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = BindSchemaRequest::decode(value.as_ref())?;
        let schema_storage = SchemaStorage::new(self.rocksdb_engine_handler.clone());
        let bind_data = SchemaResourceBind {
            resource_name: req.resource_name.clone(),
            schema_name: req.schema_name.clone(),
        };
        schema_storage.save_bind(&bind_data)?;
        Ok(())
    }

    pub fn delete_schema_bind(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = UnBindSchemaRequest::decode(value.as_ref())?;
        let schema_storage = SchemaStorage::new(self.rocksdb_engine_handler.clone());
        schema_storage.delete_bind(&req.resource_name, &req.schema_name)?;
        Ok(())
    }

    pub fn delete_offset_data(&self, _: Bytes) -> Result<(), MetaServiceError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
    use common_base::tools::unique_id;
    use common_base::utils::file_utils::test_temp_dir;
    use common_config::broker::default_broker_config;
    use metadata_struct::meta::node::BrokerNode;
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use rocksdb_engine::storage::family::column_family_list;

    use crate::core::cache::CacheManager;
    use crate::raft::route::common::DataRouteCluster;
    use crate::storage::common::node::NodeStorage;
    use prost::Message;
    use protocol::meta::meta_service_common::RegisterNodeRequest;

    #[tokio::test]
    async fn register_unregister_node() {
        let config = default_broker_config();
        let node_id = 999;
        let node_ip = "127.0.0.1".to_string();

        let node = BrokerNode {
            node_id,
            node_ip: node_ip.clone(),
            roles: Vec::new(),
            ..Default::default()
        };
        let request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };
        let data = Bytes::copy_from_slice(&RegisterNodeRequest::encode_to_vec(&request));
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let cluster_cache = Arc::new(CacheManager::new(rocksdb_engine.clone()));
        let route = DataRouteCluster::new(rocksdb_engine.clone(), cluster_cache);
        route.add_node(data).await.unwrap();

        let node_storage = NodeStorage::new(rocksdb_engine.clone());
        let node = node_storage.get(node_id).unwrap();
        let broker_node = node.unwrap();
        assert_eq!(broker_node.node_id, node_id);
        assert_eq!(broker_node.node_ip, node_ip);

        let _ = node_storage.delete(node_id);
        let res = node_storage.get(node_id).unwrap();
        assert!(res.is_none());
    }
}
