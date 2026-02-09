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

use crate::controller::call_broker::call::BrokerCallManager;
use crate::controller::call_broker::mqtt::{
    update_cache_by_add_session, update_cache_by_delete_session,
};
use crate::controller::session_expire::ExpireLastWill;
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::storage::mqtt::subscribe::MqttSubscribeStorage;
use crate::{
    raft::route::data::{StorageData, StorageDataType},
    storage::mqtt::session::MqttSessionStorage,
};
use common_base::tools::now_second;
use common_base::utils::serialize::encode_to_bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use protocol::meta::meta_service_mqtt::{
    CreateSessionReply, CreateSessionRequest, DeleteSessionReply, DeleteSessionRequest,
    DeleteSubscribeRequest, GetLastWillMessageReply, GetLastWillMessageRequest, ListSessionReply,
    ListSessionRequest, SaveLastWillMessageReply, SaveLastWillMessageRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

// Session Operations
pub fn list_session_by_req(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSessionRequest,
) -> Result<ListSessionReply, MetaServiceError> {
    let mut not_persist_session_list = read_not_persist_session(cache_manager, &req.client_id)?;
    let persist_session_list = read_persist_session(rocksdb_engine_handler, &req.client_id)?;
    not_persist_session_list.extend(persist_session_list);
    Ok(ListSessionReply {
        sessions: not_persist_session_list,
    })
}

fn read_not_persist_session(
    cache_manager: &Arc<CacheManager>,
    client_id: &str,
) -> Result<Vec<Vec<u8>>, MetaServiceError> {
    let mut sessions = Vec::new();
    if !client_id.is_empty() {
        if let Some(session) = cache_manager.get_session(client_id) {
            sessions.push(session.encode()?);
        }
    } else {
        sessions = cache_manager
            .session_list
            .clone()
            .into_iter()
            .map(|(_, raw)| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
    }
    Ok(sessions)
}

fn read_persist_session(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    client_id: &str,
) -> Result<Vec<Vec<u8>>, MetaServiceError> {
    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    let mut sessions = Vec::new();

    if !client_id.is_empty() {
        if let Some(data) = storage.get(client_id)? {
            sessions.push(data.encode()?);
        }
    } else {
        let data = storage.list_all()?;
        sessions = data
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
    }
    Ok(sessions)
}

pub async fn create_session_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateSessionRequest,
) -> Result<CreateSessionReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::MqttSetSession, encode_to_bytes(req));
    raft_manager.write_mqtt(data).await?;

    let session = MqttSession::decode(&req.session)?;
    update_cache_by_add_session(call_manager, client_pool, session).await?;

    Ok(CreateSessionReply {})
}

pub async fn delete_session_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    cache_manager: &Arc<CacheManager>,
    req: &DeleteSessionRequest,
) -> Result<DeleteSessionReply, MetaServiceError> {
    let session_storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    let session = if let Some(session) = session_storage.get(&req.client_id)? {
        session
    } else if let Some(session) = cache_manager.get_session(&req.client_id) {
        session
    } else {
        return Ok(DeleteSessionReply::default());
    };

    let data = StorageData::new(StorageDataType::MqttDeleteSession, encode_to_bytes(req));
    raft_manager.write_mqtt(data).await?;

    // delete subscribe
    let storage = MqttSubscribeStorage::new(rocksdb_engine_handler.clone());
    let subscribes = storage.list_by_client_id(&req.client_id)?;
    if !subscribes.is_empty() {
        let request = DeleteSubscribeRequest {
            client_id: req.client_id.clone(),
            ..Default::default()
        };
        let data = StorageData::new(
            StorageDataType::MqttDeleteSubscribe,
            encode_to_bytes(&request),
        );
        raft_manager.write_metadata(data).await?;
    }

    // Check for last will message and schedule expiration
    let will_storage = MqttLastWillStorage::new(rocksdb_engine_handler.clone());
    if let Some(will_message) = will_storage.get(&req.client_id)? {
        let delay = session.last_will_delay_interval.unwrap_or_default();
        cache_manager.add_expire_last_will(ExpireLastWill {
            client_id: will_message.client_id,
            delay_sec: now_second() + delay,
        });
    }

    update_cache_by_delete_session(call_manager, client_pool, session.clone()).await?;

    Ok(DeleteSessionReply {})
}

// Last Will Message Operations
pub async fn save_last_will_message_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &SaveLastWillMessageRequest,
) -> Result<SaveLastWillMessageReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttSaveLastWillMessage,
        encode_to_bytes(req),
    );
    raft_manager.write_mqtt(data).await?;

    Ok(SaveLastWillMessageReply {})
}

pub async fn get_last_will_message_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetLastWillMessageRequest,
) -> Result<GetLastWillMessageReply, MetaServiceError> {
    let storage = MqttLastWillStorage::new(rocksdb_engine_handler.clone());

    let message = storage
        .get(&req.client_id)?
        .map(|will| will.encode())
        .transpose()?
        .unwrap_or_default();

    Ok(GetLastWillMessageReply { message })
}
