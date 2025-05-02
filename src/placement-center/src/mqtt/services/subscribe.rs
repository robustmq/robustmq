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

use crate::{
    core::error::PlacementCenterError,
    mqtt::controller::call_broker::{
        update_cache_by_add_subscribe, update_cache_by_delete_subscribe, MQTTInnerCallManager,
    },
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
    storage::mqtt::subscribe::MqttSubscribeStorage,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::{
    DeleteAutoSubscribeRuleRequest, DeleteSubscribeRequest, ListAutoSubscribeRuleRequest,
    ListSubscribeRequest, SetAutoSubscribeRuleRequest, SetSubscribeRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;
use tonic::Request;
use tracing::warn;

pub async fn delete_subscribe_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteSubscribeRequest>,
) -> Result<(), PlacementCenterError> {
    let req = request.into_inner();

    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    let subscribes = if !req.path.is_empty() {
        match storage.get(&req.cluster_name, &req.client_id, &req.path)? {
            Some(subscribe) => vec![subscribe],
            None => {
                return Err(PlacementCenterError::SubscribeDoesNotExist(req.path));
            }
        }
    } else {
        storage.list_by_client_id(&req.cluster_name, &req.client_id)?
    };

    if subscribes.is_empty() {
        return Ok(());
    }

    let data = StorageData::new(
        StorageDataType::MqttDeleteSubscribe,
        DeleteSubscribeRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    for raw in subscribes {
        update_cache_by_delete_subscribe(&req.cluster_name, mqtt_call_manager, client_pool, raw)
            .await?;
    }
    Ok(())
}

pub fn list_subscribe_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListSubscribeRequest>,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
    let req = request.into_inner();
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    let data = storage.list_by_cluster(&req.cluster_name)?;
    let subscribes = data.into_iter().map(|raw| raw.encode()).collect();

    Ok(subscribes)
}

pub async fn set_subscribe_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<SetSubscribeRequest>,
) -> Result<(), PlacementCenterError> {
    let req = request.into_inner();

    let data = StorageData::new(
        StorageDataType::MqttSetSubscribe,
        SetSubscribeRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let subscribe = match serde_json::from_slice::<MqttSubscribe>(&req.subscribe) {
        Ok(subscribe) => subscribe,
        Err(e) => {
            warn!("set subscribe error:{}", e);
            return Err(PlacementCenterError::CommonError(e.to_string()));
        }
    };

    update_cache_by_add_subscribe(&req.cluster_name, mqtt_call_manager, client_pool, subscribe)
        .await?;

    Ok(())
}

pub async fn set_auto_subscribe_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<SetAutoSubscribeRuleRequest>,
) -> Result<(), PlacementCenterError> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttSetAutoSubscribeRule,
        SetAutoSubscribeRuleRequest::encode_to_vec(&req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(())
}

pub async fn delete_auto_subscribe_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<DeleteAutoSubscribeRuleRequest>,
) -> Result<(), PlacementCenterError> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttDeleteAutoSubscribeRule,
        DeleteAutoSubscribeRuleRequest::encode_to_vec(&req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(())
}

pub fn list_auto_subscribe_rule_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListAutoSubscribeRuleRequest>,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
    let req = request.into_inner();
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    let data = storage.list_auto_subscribe_rule(&req.cluster_name)?;

    let result: Vec<Vec<u8>> = data.into_iter().map(|raw| raw.encode()).collect();

    Ok(result)
}
