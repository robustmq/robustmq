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
use log::warn;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::{
    DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest, DeleteSubscribeReply,
    DeleteSubscribeRequest, ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest,
    ListSubscribeReply, ListSubscribeRequest, SetAutoSubscribeRuleReply,
    SetAutoSubscribeRuleRequest, SetSubscribeReply, SetSubscribeRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn delete_subscribe_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteSubscribeRequest>,
) -> Result<Response<DeleteSubscribeReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttDeleteSubscribe,
        DeleteSubscribeRequest::encode_to_vec(&req),
    );
    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };

    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    if !req.path.is_empty() {
        let subscribe = match storage.get(&req.cluster_name, &req.client_id, &req.path) {
            Ok(Some(subscribe)) => subscribe,
            Ok(None) => {
                return Err(Status::cancelled(
                    PlacementCenterError::SubscribeDoesNotExist(req.path).to_string(),
                ));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };

        if let Err(e) = update_cache_by_delete_subscribe(
            &req.cluster_name,
            mqtt_call_manager,
            client_pool,
            subscribe,
        )
        .await
        {
            return Err(Status::cancelled(e.to_string()));
        };
    } else {
        let subscribes = storage.list_by_client_id(&req.cluster_name, &req.client_id)?;
        for raw in subscribes {
            if let Err(e) = update_cache_by_delete_subscribe(
                &req.cluster_name,
                mqtt_call_manager,
                client_pool,
                raw,
            )
            .await
            {
                return Err(Status::cancelled(e.to_string()));
            };
        }
    }
    Ok(Response::new(DeleteSubscribeReply::default()))
}

pub fn list_subscribe_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListSubscribeRequest>,
) -> Result<Response<ListSubscribeReply>, Status> {
    let req = request.into_inner();
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    match storage.list_by_cluster(&req.cluster_name) {
        Ok(data) => {
            let mut subscribes = Vec::new();
            for raw in data {
                subscribes.push(raw.encode());
            }
            Ok(Response::new(ListSubscribeReply { subscribes }))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn set_subscribe_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<SetSubscribeRequest>,
) -> Result<Response<SetSubscribeReply>, Status> {
    let req = request.into_inner();
    let raft_machine_apply = &raft_machine_apply;
    let call_manager = &mqtt_call_manager;
    let client_pool = &client_pool;
    let data = StorageData::new(
        StorageDataType::MqttSetSubscribe,
        SetSubscribeRequest::encode_to_vec(&req),
    );
    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    }

    let subscribe = match serde_json::from_slice::<MqttSubscribe>(&req.subscribe) {
        Ok(subscribe) => subscribe,
        Err(e) => {
            warn!("set subscribe error:{}", e);
            return Err(Status::cancelled(e.to_string()));
        }
    };

    if let Err(e) =
        update_cache_by_add_subscribe(&req.cluster_name, call_manager, client_pool, subscribe).await
    {
        return Err(Status::cancelled(e.to_string()));
    }
    Ok(Response::new(SetSubscribeReply::default()))
}

pub async fn set_auto_subscribe_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<SetAutoSubscribeRuleRequest>,
) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttSetAutoSubscribeRule,
        SetAutoSubscribeRuleRequest::encode_to_vec(&req),
    );

    match raft_machine_apply.client_write(data).await {
        Ok(_) => Ok(Response::new(SetAutoSubscribeRuleReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn delete_auto_subscribe_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<DeleteAutoSubscribeRuleRequest>,
) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttDeleteAutoSubscribeRule,
        DeleteAutoSubscribeRuleRequest::encode_to_vec(&req),
    );

    match raft_machine_apply.client_write(data).await {
        Ok(_) => Ok(Response::new(DeleteAutoSubscribeRuleReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub fn list_auto_subscribe_rule_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListAutoSubscribeRuleRequest>,
) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
    let req = request.into_inner();
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    match storage.list_auto_subscribe_rule(&req.cluster_name) {
        Ok(data) => {
            let mut result = Vec::new();
            for raw in data {
                result.push(raw.encode());
            }
            Ok(Response::new(ListAutoSubscribeRuleReply {
                auto_subscribe_rules: result,
            }))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}
