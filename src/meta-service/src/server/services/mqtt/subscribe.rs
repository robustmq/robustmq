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
    controller::call_broker::mqtt::{
        update_cache_by_add_subscribe, update_cache_by_delete_subscribe, BrokerCallManager,
    },
    core::error::MetaServiceError,
    raft::{
        manager::MultiRaftManager,
        route::data::{StorageData, StorageDataType},
    },
    storage::mqtt::subscribe::MqttSubscribeStorage,
};
use common_base::utils::serialize::encode_to_bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use protocol::meta::meta_service_mqtt::{
    DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest, DeleteSubscribeReply,
    DeleteSubscribeRequest, ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest,
    ListSubscribeReply, ListSubscribeRequest, SetAutoSubscribeRuleReply,
    SetAutoSubscribeRuleRequest, SetSubscribeReply, SetSubscribeRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

// Subscribe Operations
pub async fn delete_subscribe_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteSubscribeRequest,
) -> Result<DeleteSubscribeReply, MetaServiceError> {
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    let subscribes = if !req.path.is_empty() {
        match storage.get(&req.client_id, &req.path)? {
            Some(subscribe) => vec![subscribe],
            None => {
                return Err(MetaServiceError::SubscribeDoesNotExist(req.path.clone()));
            }
        }
    } else {
        storage.list_by_client_id(&req.client_id)?
    };

    if subscribes.is_empty() {
        return Ok(DeleteSubscribeReply {});
    }

    let data = StorageData::new(StorageDataType::MqttDeleteSubscribe, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    for raw in subscribes {
        update_cache_by_delete_subscribe(call_manager, client_pool, raw).await?;
    }

    Ok(DeleteSubscribeReply {})
}

pub fn list_subscribe_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    _req: &ListSubscribeRequest,
) -> Result<ListSubscribeReply, MetaServiceError> {
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    let subscribes = storage
        .list_all()?
        .into_iter()
        .map(|raw| raw.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListSubscribeReply { subscribes })
}

pub async fn set_subscribe_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &SetSubscribeRequest,
) -> Result<SetSubscribeReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::MqttSetSubscribe, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let subscribe = MqttSubscribe::decode(&req.subscribe)
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))?;
    update_cache_by_add_subscribe(call_manager, client_pool, subscribe).await?;

    Ok(SetSubscribeReply {})
}

// Auto Subscribe Rule Operations
pub async fn set_auto_subscribe_rule_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &SetAutoSubscribeRuleRequest,
) -> Result<SetAutoSubscribeRuleReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttSetAutoSubscribeRule,
        encode_to_bytes(req),
    );
    raft_manager.write_metadata(data).await?;

    Ok(SetAutoSubscribeRuleReply {})
}

pub async fn delete_auto_subscribe_rule_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &DeleteAutoSubscribeRuleRequest,
) -> Result<DeleteAutoSubscribeRuleReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteAutoSubscribeRule,
        encode_to_bytes(req),
    );
    raft_manager.write_metadata(data).await?;

    Ok(DeleteAutoSubscribeRuleReply {})
}

pub fn list_auto_subscribe_rule_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    _req: &ListAutoSubscribeRuleRequest,
) -> Result<ListAutoSubscribeRuleReply, MetaServiceError> {
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    let data = storage.list_all_auto_subscribe_rules()?;

    let auto_subscribe_rules = data
        .into_iter()
        .map(|raw| raw.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListAutoSubscribeRuleReply {
        auto_subscribe_rules,
    })
}
