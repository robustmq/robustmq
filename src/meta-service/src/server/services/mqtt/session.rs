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
    update_cache_by_add_session, update_cache_by_delete_session, MQTTInnerCallManager,
};
use crate::controller::mqtt::session_expire::ExpireLastWill;
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::{
    raft::route::data::{StorageData, StorageDataType},
    storage::mqtt::session::MqttSessionStorage,
};
use bytes::Bytes;
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use prost::Message;
use protocol::meta::meta_service_mqtt::{
    CreateSessionReply, CreateSessionRequest, DeleteSessionReply, DeleteSessionRequest,
    GetLastWillMessageReply, GetLastWillMessageRequest, ListSessionReply, ListSessionRequest,
    SaveLastWillMessageReply, SaveLastWillMessageRequest, UpdateSessionReply, UpdateSessionRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

// Session
pub fn list_session_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSessionRequest,
) -> Result<ListSessionReply, MetaServiceError> {
    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    let mut sessions = Vec::new();

    if !req.client_id.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.client_id)? {
            sessions.push(data.encode()?);
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
        sessions = data
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
    }

    Ok(ListSessionReply { sessions })
}

pub async fn create_session_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateSessionRequest,
) -> Result<CreateSessionReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttSetSession,
        Bytes::copy_from_slice(&CreateSessionRequest::encode_to_vec(req)),
    );

    raft_manager.write_mqtt(data).await?;

    let session = MqttSession::decode(&req.session)?;

    update_cache_by_add_session(&req.cluster_name, call_manager, client_pool, session).await?;

    Ok(CreateSessionReply {})
}

pub async fn update_session_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &UpdateSessionRequest,
) -> Result<UpdateSessionReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttUpdateSession,
        Bytes::copy_from_slice(&UpdateSessionRequest::encode_to_vec(req)),
    );
    raft_manager.write_mqtt(data).await?;

    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    if let Some(session) = storage.get(&req.cluster_name, &req.client_id)? {
        update_cache_by_add_session(&req.cluster_name, call_manager, client_pool, session).await?;
    }
    Ok(UpdateSessionReply {})
}

pub async fn delete_session_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    cache_manager: &Arc<CacheManager>,
    req: &DeleteSessionRequest,
) -> Result<DeleteSessionReply, MetaServiceError> {
    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    let session = storage
        .get(&req.cluster_name, &req.client_id)?
        .ok_or_else(|| MetaServiceError::SessionDoesNotExist(req.client_id.clone()))?;

    let storage = MqttLastWillStorage::new(rocksdb_engine_handler.clone());
    match storage.get(&req.cluster_name, &req.client_id) {
        Ok(Some(will_message)) => {
            let delay = session.last_will_delay_interval.unwrap_or_default();
            cache_manager.add_expire_last_will(ExpireLastWill {
                client_id: will_message.client_id.clone(),
                delay_sec: now_second() + delay,
                cluster_name: req.cluster_name.to_owned(),
            });
        }
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    let data = StorageData::new(
        StorageDataType::MqttDeleteSession,
        Bytes::copy_from_slice(&DeleteSessionRequest::encode_to_vec(req)),
    );

    raft_manager.write_mqtt(data).await?;

    update_cache_by_delete_session(
        &req.cluster_name,
        call_manager,
        client_pool,
        session.clone(),
    )
    .await?;
    Ok(DeleteSessionReply {})
}

// Will Message
pub async fn save_last_will_message_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &SaveLastWillMessageRequest,
) -> Result<SaveLastWillMessageReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttSaveLastWillMessage,
        Bytes::copy_from_slice(&SaveLastWillMessageRequest::encode_to_vec(req)),
    );

    raft_manager.write_mqtt(data).await?;
    Ok(SaveLastWillMessageReply {})
}

pub async fn get_last_will_message_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetLastWillMessageRequest,
) -> Result<GetLastWillMessageReply, MetaServiceError> {
    let storage = MqttLastWillStorage::new(rocksdb_engine_handler.clone());
    if let Some(will) = storage.get(&req.cluster_name, &req.client_id)? {
        return Ok(GetLastWillMessageReply {
            message: will.encode()?,
        });
    }

    Ok(GetLastWillMessageReply {
        message: Vec::new(),
    })
}
