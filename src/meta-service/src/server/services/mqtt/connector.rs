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

use crate::controller::mqtt::call_broker::{
    update_cache_by_delete_connector, MQTTInnerCallManager,
};
use crate::controller::mqtt::connector::status::ConnectorContext;
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::mqtt::connector::MqttConnectorStorage;
use common_base::utils::serialize::encode_to_bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use protocol::meta::meta_service_mqtt::{
    ConnectorHeartbeatReply, ConnectorHeartbeatRequest, CreateConnectorReply,
    CreateConnectorRequest, DeleteConnectorReply, DeleteConnectorRequest, ListConnectorReply,
    ListConnectorRequest, UpdateConnectorReply, UpdateConnectorRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConnectorHeartbeat {
    pub cluster_name: String,
    pub connector_name: String,
    pub last_heartbeat: u64,
}

// Connector Heartbeat
pub fn connector_heartbeat_by_req(
    cache_manager: &Arc<CacheManager>,
    req: &ConnectorHeartbeatRequest,
) -> Result<ConnectorHeartbeatReply, MetaServiceError> {
    for raw in &req.heatbeats {
        if let Some(connector) = cache_manager.get_connector(&req.cluster_name, &raw.connector_name)
        {
            if connector.broker_id.is_none() {
                warn!("connector:{} not register", raw.connector_name);
                continue;
            }

            if connector.broker_id.unwrap() != raw.broker_id {
                warn!("connector:{} not register", raw.connector_name);
                continue;
            }

            cache_manager.report_connector_heartbeat(
                &req.cluster_name,
                &raw.connector_name,
                raw.heartbeat_time,
            );
        }
    }
    Ok(ConnectorHeartbeatReply {})
}

// Connector Operations
pub fn list_connectors_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListConnectorRequest,
) -> Result<ListConnectorReply, MetaServiceError> {
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());

    let connectors = if !req.connector_name.is_empty() {
        match storage.get(&req.cluster_name, &req.connector_name)? {
            Some(data) => vec![data.encode()?],
            None => Vec::new(),
        }
    } else {
        storage
            .list(&req.cluster_name)?
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(ListConnectorReply { connectors })
}

pub async fn create_connector_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    req: &CreateConnectorRequest,
) -> Result<CreateConnectorReply, MetaServiceError> {
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());

    // Check if connector already exists
    if storage
        .get(&req.cluster_name, &req.connector_name)?
        .is_some()
    {
        return Err(MetaServiceError::ConnectorAlreadyExist(
            req.connector_name.clone(),
        ));
    }

    let connector = MQTTConnector::decode(&req.connector)?;
    let ctx = ConnectorContext::new(
        raft_manager.clone(),
        mqtt_call_manager.clone(),
        client_pool.clone(),
        cache_manager.clone(),
    );
    ctx.save_connector(connector).await?;

    Ok(CreateConnectorReply {})
}

pub async fn update_connector_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    req: &UpdateConnectorRequest,
) -> Result<UpdateConnectorReply, MetaServiceError> {
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());

    // Check if connector exists
    if storage
        .get(&req.cluster_name, &req.connector_name)?
        .is_none()
    {
        return Err(MetaServiceError::ConnectorNotFound(
            req.connector_name.clone(),
        ));
    }

    let connector = MQTTConnector::decode(&req.connector)?;
    let ctx = ConnectorContext::new(
        raft_manager.clone(),
        mqtt_call_manager.clone(),
        client_pool.clone(),
        cache_manager.clone(),
    );
    ctx.save_connector(connector).await?;

    Ok(UpdateConnectorReply {})
}

pub async fn delete_connector_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteConnectorRequest,
) -> Result<DeleteConnectorReply, MetaServiceError> {
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());

    // Get connector to delete (must exist)
    let connector = storage
        .get(&req.cluster_name, &req.connector_name)?
        .ok_or_else(|| MetaServiceError::ConnectorNotFound(req.connector_name.clone()))?;

    let data = StorageData::new(StorageDataType::MqttDeleteConnector, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    update_cache_by_delete_connector(&req.cluster_name, mqtt_call_manager, client_pool, connector)
        .await?;

    Ok(DeleteConnectorReply {})
}
