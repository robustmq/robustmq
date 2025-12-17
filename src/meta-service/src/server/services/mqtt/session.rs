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

use crate::controller::call_broker::mqtt::{
    update_cache_by_add_session, update_cache_by_delete_session, BrokerCallManager,
};
use crate::controller::session_expire::ExpireLastWill;
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
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
    GetLastWillMessageReply, GetLastWillMessageRequest, ListSessionReply, ListSessionRequest,
    SaveLastWillMessageReply, SaveLastWillMessageRequest, UpdateSessionReply, UpdateSessionRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

// Session Operations
pub fn list_session_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSessionRequest,
) -> Result<ListSessionReply, MetaServiceError> {
    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    let mut sessions = Vec::new();

    if !req.client_id.is_empty() {
        if let Some(data) = storage.get(&req.client_id)? {
            sessions.push(data.encode()?);
        }
    } else {
        let data = storage.list_all()?;
        sessions = data
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
    }

    Ok(ListSessionReply { sessions })
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

pub async fn update_session_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &UpdateSessionRequest,
) -> Result<UpdateSessionReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::MqttUpdateSession, encode_to_bytes(req));
    raft_manager.write_mqtt(data).await?;

    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    if let Some(session) = storage.get(&req.client_id)? {
        update_cache_by_add_session(call_manager, client_pool, session).await?;
    }

    Ok(UpdateSessionReply {})
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
    let session = session_storage
        .get(&req.client_id)?
        .ok_or_else(|| MetaServiceError::SessionDoesNotExist(req.client_id.clone()))?;

    // Check for last will message and schedule expiration
    let will_storage = MqttLastWillStorage::new(rocksdb_engine_handler.clone());
    if let Some(will_message) = will_storage.get(&req.client_id)? {
        let delay = session.last_will_delay_interval.unwrap_or_default();
        cache_manager.add_expire_last_will(ExpireLastWill {
            client_id: will_message.client_id,
            delay_sec: now_second() + delay,
        });
    }

    let data = StorageData::new(StorageDataType::MqttDeleteSession, encode_to_bytes(req));
    raft_manager.write_mqtt(data).await?;

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
