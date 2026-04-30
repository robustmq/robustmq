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

use crate::core::error::MetaServiceError;
use crate::core::notify::{send_notify_by_add_session, send_notify_by_delete_session};
use crate::raft::manager::MultiRaftManager;
use crate::storage::mqtt::subscribe::MqttSubscribeStorage;
use crate::{
    raft::route::data::{StorageData, StorageDataType},
    storage::mqtt::session::MqttSessionStorage,
};
use broker_core::cache::NodeCacheManager;
use common_base::tools::now_second;
use common_base::utils::serialize::encode_to_bytes;
use delay_task::manager::DelayTaskManager;
use delay_task::{DelayTask, DelayTaskData};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use node_call::NodeCallManager;
use protocol::meta::meta_service_mqtt::{
    CreateSessionReply, CreateSessionRequest, DeleteSessionReply, DeleteSessionRequest,
    DeleteSubscribeRequest, ListSessionReply, ListSessionRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

type ListSessionStream =
    Result<Pin<Box<dyn Stream<Item = Result<ListSessionReply, Status>> + Send>>, MetaServiceError>;

// Session Operations
pub fn list_session_by_req(
    node_cache_manager: &Arc<NodeCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSessionRequest,
) -> ListSessionStream {
    let mut sessions = read_not_persist_session(node_cache_manager, &req.tenant, &req.client_id)?;
    let persist_sessions =
        read_persist_session(rocksdb_engine_handler, &req.tenant, &req.client_id)?;
    sessions.extend(persist_sessions);

    let output = async_stream::try_stream! {
        for session in sessions {
            yield ListSessionReply { session };
        }
    };

    Ok(Box::pin(output))
}

fn read_not_persist_session(
    node_cache_manager: &Arc<NodeCacheManager>,
    tenant: &str,
    client_id: &str,
) -> Result<Vec<Vec<u8>>, MetaServiceError> {
    if !client_id.is_empty() {
        if let Some(session) = node_cache_manager.get_session(tenant, client_id) {
            return Ok(vec![session.encode()?]);
        }
        return Ok(vec![]);
    }

    if !tenant.is_empty() {
        let sessions = node_cache_manager
            .list_sessions_by_tenant(tenant)
            .into_iter()
            .map(|s| s.encode())
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(sessions);
    }

    let sessions = node_cache_manager
        .session_list
        .iter()
        .map(|e| e.value().encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sessions)
}

fn read_persist_session(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    tenant: &str,
    client_id: &str,
) -> Result<Vec<Vec<u8>>, MetaServiceError> {
    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    let mut sessions = Vec::new();

    if !client_id.is_empty() {
        if let Some(data) = storage.get(tenant, client_id)? {
            sessions.push(data.encode()?);
        }
    } else if !tenant.is_empty() {
        let data = storage.list_by_tenant(tenant)?;
        sessions = data
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
    } else {
        let data = storage.list()?;
        sessions = data
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
    }
    Ok(sessions)
}

pub async fn create_session_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    _client_pool: &Arc<ClientPool>,
    req: &CreateSessionRequest,
) -> Result<CreateSessionReply, MetaServiceError> {
    let routing_key = req
        .sessions
        .first()
        .map(|s| s.client_id.as_str())
        .unwrap_or("");

    let data = StorageData::new(StorageDataType::MqttSetSession, encode_to_bytes(req));
    raft_manager.write_data(routing_key, data).await?;

    for raw in &req.sessions {
        let session = MqttSession::decode(&raw.session)?;
        send_notify_by_add_session(call_manager, session).await?;
    }

    Ok(CreateSessionReply {})
}

pub async fn delete_session_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    delay_task_manager: &Arc<DelayTaskManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    node_cache_manager: &Arc<NodeCacheManager>,
    req: &DeleteSessionRequest,
) -> Result<DeleteSessionReply, MetaServiceError> {
    let session_storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    let session = if let Some(session) = session_storage.get(&req.tenant, &req.client_id)? {
        session
    } else if let Some(session) = node_cache_manager.get_session(&req.tenant, &req.client_id) {
        session
    } else {
        return Ok(DeleteSessionReply::default());
    };

    let data = StorageData::new(StorageDataType::MqttDeleteSession, encode_to_bytes(req));
    raft_manager.write_data(&req.client_id, data).await?;

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
    if session.is_contain_last_will {
        let delay = session.last_will_delay_interval.unwrap_or_default();
        delay_task_manager
            .create_task(DelayTask::build_persistent(
                req.client_id.clone(),
                DelayTaskData::MQTTLastwillExpire(req.client_id.clone()),
                now_second() + delay,
            ))
            .await?;
    }
    send_notify_by_delete_session(call_manager, session.clone()).await?;

    Ok(DeleteSessionReply {})
}
