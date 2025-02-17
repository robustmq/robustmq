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

use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::{
    CreateConnectorRequest, DeleteConnectorRequest, ListConnectorRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

use crate::{
    core::error::PlacementCenterError,
    mqtt::controller::call_broker::{
        update_cache_by_add_connector, update_cache_by_delete_connector, MQTTInnerCallManager,
    },
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
    storage::mqtt::connector::MqttConnectorStorage,
};

pub async fn list_connector_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: ListConnectorRequest,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
    let storage = MqttConnectorStorage::new(rocksdb_engine_handler.clone());

    if !req.connector_name.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.connector_name)? {
            return Ok(vec![data.encode()]);
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
        let mut result = Vec::new();
        for raw in data {
            result.push(raw.encode());
        }
        return Ok(result);
    }
    Ok(Vec::new())
}

pub async fn create_connector_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: CreateConnectorRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttSetConnector,
        CreateConnectorRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let connector = serde_json::from_slice::<MQTTConnector>(&req.connector)?;
    update_cache_by_add_connector(&req.cluster_name, call_manager, client_pool, connector).await?;

    start_connector().await;
    Ok(())
}

pub async fn delete_connector_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: DeleteConnectorRequest,
) -> Result<(), PlacementCenterError> {
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
        call_manager,
        client_pool,
        connector.unwrap(),
    )
    .await?;

    stop_connector().await;
    Ok(())
}

async fn start_connector() {}

async fn stop_connector() {}
