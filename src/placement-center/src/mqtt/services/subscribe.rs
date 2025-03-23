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
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::{
    DeleteAutoSubscribeRuleRequest, DeleteSubscribeRequest, ListAutoSubscribeRuleRequest,
    SetAutoSubscribeRuleRequest, SetSubscribeRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

pub async fn save_subscribe_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: SetSubscribeRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttSetSubscribe,
        SetSubscribeRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let subscribe = serde_json::from_slice::<MqttSubscribe>(&req.subscribe)?;
    update_cache_by_add_subscribe(&req.cluster_name, call_manager, client_pool, subscribe).await?;
    Ok(())
}

pub async fn delete_subscribe_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: DeleteSubscribeRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteSubscribe,
        DeleteSubscribeRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    if !req.path.is_empty() {
        if let Some(subscribe) = storage.get(&req.cluster_name, &req.client_id, &req.path)? {
            update_cache_by_delete_subscribe(
                &req.cluster_name,
                call_manager,
                client_pool,
                subscribe,
            )
            .await?;
        }
    } else {
        let subscribes = storage.list_by_client_id(&req.cluster_name, &req.client_id)?;
        for raw in subscribes {
            update_cache_by_delete_subscribe(&req.cluster_name, call_manager, client_pool, raw)
                .await?;
        }
    }
    Ok(())
}

pub async fn set_auto_subscribe_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: SetAutoSubscribeRuleRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttSetAutoSubscribeRule,
        SetAutoSubscribeRuleRequest::encode_to_vec(&req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(())
}

pub async fn delete_auto_subscribe_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: DeleteAutoSubscribeRuleRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteAutoSubscribeRule,
        DeleteAutoSubscribeRuleRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;
    Ok(())
}

pub async fn list_auto_subscribe_rule_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: ListAutoSubscribeRuleRequest,
) -> Result<Vec<MqttAutoSubscribeRule>, PlacementCenterError> {
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    let subscribes = storage.list_auto_subscribe_rule(&req.cluster_name)?;
    Ok(subscribes)
}
