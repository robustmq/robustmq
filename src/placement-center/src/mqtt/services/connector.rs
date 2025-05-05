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

use crate::core::error::PlacementCenterError;
use crate::mqtt::cache::MqttCacheManager;
use crate::mqtt::connector::status::save_connector;
use crate::mqtt::controller::call_broker::{
    update_cache_by_delete_connector, MQTTInnerCallManager,
};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::mqtt::connector::MqttConnectorStorage;
use grpc_clients::pool::ClientPool;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::{
    ConnectorHeartbeatRequest, CreateConnectorRequest, DeleteConnectorRequest,
    ListConnectorRequest, UpdateConnectorRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;
use tonic::Request;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct ConnectorHeartbeat {
    pub cluster_name: String,
    pub connector_name: String,
    pub last_heartbeat: u64,
}

pub fn connector_heartbeat_by_req(
    mqtt_cache: &Arc<MqttCacheManager>,
    request: Request<ConnectorHeartbeatRequest>,
) -> Result<(), PlacementCenterError> {
    let req = request.into_inner();
    for raw in req.heatbeats {
        if let Some(connector) = mqtt_cache.get_connector(&req.cluster_name, &raw.connector_name) {
            if connector.broker_id.is_none() {
                warn!("connector:{} not register", raw.connector_name);
                continue;
            }

            if connector.broker_id.unwrap() != raw.broker_id {
                warn!("connector:{} not register", raw.connector_name);
                continue;
            }

            mqtt_cache.report_connector_heartbeat(
                &req.cluster_name,
                &raw.connector_name,
                raw.heartbeat_time,
            );
        }
    }
    Ok(())
}

pub fn list_connectors_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListConnectorRequest>,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
    let req = request.into_inner();
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());

    if !req.connector_name.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.connector_name)? {
            return Ok(vec![data.encode()]);
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
        let connectors = data.into_iter().map(|raw| raw.encode()).collect();
        return Ok(connectors);
    }

    Ok(Vec::new())
}

pub async fn create_connector_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateConnectorRequest>,
) -> Result<(), PlacementCenterError> {
    let req = request.into_inner();
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());
    let connector = storage.get(&req.cluster_name, &req.connector_name)?;

    if connector.is_some() {
        return Err(PlacementCenterError::ConnectorAlreadyExist(
            req.connector_name,
        ));
    }

    save_connector(raft_machine_apply, req, mqtt_call_manager, client_pool).await?;

    Ok(())
}

pub async fn update_connector_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<UpdateConnectorRequest>,
) -> Result<(), PlacementCenterError> {
    let req = request.into_inner();
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());
    let connector = storage.get(&req.cluster_name, &req.connector_name)?;

    if connector.is_none() {
        return Err(PlacementCenterError::ConnectorNotFound(req.connector_name));
    }

    let create_req = CreateConnectorRequest {
        cluster_name: req.cluster_name.clone(),
        connector_name: req.connector_name.clone(),
        connector: req.connector.clone(),
    };

    save_connector(
        raft_machine_apply,
        create_req,
        mqtt_call_manager,
        client_pool,
    )
    .await?;

    Ok(())
}

pub async fn delete_connector_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteConnectorRequest>,
) -> Result<(), PlacementCenterError> {
    let req = request.into_inner();
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());
    let connector = storage.get(&req.cluster_name, &req.connector_name)?;

    if connector.is_none() {
        return Err(PlacementCenterError::ConnectorNotFound(req.connector_name));
    }

    let data = StorageData::new(
        StorageDataType::MqttDeleteConnector,
        DeleteConnectorRequest::encode_to_vec(&req),
    );

    raft_machine_apply.client_write(data).await?;

    update_cache_by_delete_connector(
        &req.cluster_name,
        mqtt_call_manager,
        client_pool,
        connector.unwrap(),
    )
    .await?;

    Ok(())
}
