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
use crate::mqtt::controller::session_expire::ExpireLastWill;
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::{
    mqtt::controller::call_broker::{
        update_cache_by_add_session, update_cache_by_delete_session, MQTTInnerCallManager,
    },
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
    storage::mqtt::session::MqttSessionStorage,
};
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::{
    CreateSessionReply, CreateSessionRequest, DeleteSessionReply, DeleteSessionRequest,
    ListSessionReply, ListSessionRequest, UpdateSessionReply, UpdateSessionRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub fn list_session_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListSessionRequest>,
) -> Result<Response<ListSessionReply>, Status> {
    let req = request.into_inner();
    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());

    if !req.client_id.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.client_id)? {
            let sessions = vec![data.encode()];
            return Ok(Response::new(ListSessionReply { sessions }));
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
        let mut sessions = Vec::new();
        for raw in data {
            sessions.push(raw.data);
        }
        return Ok(Response::new(ListSessionReply { sessions }));
    }
    Ok(Response::new(ListSessionReply {
        sessions: Vec::new(),
    }))
}

pub async fn create_session_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateSessionRequest>,
) -> Result<Response<CreateSessionReply>, Status> {
    let req: CreateSessionRequest = request.into_inner();

    let data = StorageData::new(
        StorageDataType::MqttSetSession,
        CreateSessionRequest::encode_to_vec(&req),
    );

    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };

    let session = match serde_json::from_str::<MqttSession>(&req.session) {
        Ok(session) => session,
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    };
    if let Err(e) =
        update_cache_by_add_session(&req.cluster_name, call_manager, client_pool, session).await
    {
        return Err(Status::cancelled(e.to_string()));
    };

    Ok(Response::new(CreateSessionReply::default()))
}

pub async fn update_session_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<UpdateSessionRequest>,
) -> Result<Response<UpdateSessionReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttUpdateSession,
        UpdateSessionRequest::encode_to_vec(&req),
    );
    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };

    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    if let Some(session) = storage.get(&req.cluster_name, &req.client_id)? {
        if let Err(e) =
            update_cache_by_delete_session(&req.cluster_name, call_manager, client_pool, session)
                .await
        {
            return Err(Status::cancelled(e.to_string()));
        };
    }
    Ok(Response::new(UpdateSessionReply::default()))
}

pub async fn delete_session_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    mqtt_cache_manager: &Arc<MqttCacheManager>,
    request: Request<DeleteSessionRequest>,
) -> Result<Response<DeleteSessionReply>, Status> {
    let req = request.into_inner();

    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    let session_opt = storage.get(&req.cluster_name, &req.client_id)?;
    if session_opt.is_none() {
        return Err(Status::cancelled(
            PlacementCenterError::SessionDoesNotExist(req.client_id).to_string(),
        ));
    }

    let session = session_opt.unwrap();
    if let Err(e) = update_cache_by_delete_session(
        &req.cluster_name,
        call_manager,
        client_pool,
        session.clone(),
    )
    .await
    {
        return Err(Status::cancelled(e.to_string()));
    }

    let storage = MqttLastWillStorage::new(rocksdb_engine_handler.clone());
    match storage.get(&req.cluster_name, &req.client_id) {
        Ok(Some(will_message)) => {
            let delay = session.last_will_delay_interval.unwrap_or_default();
            mqtt_cache_manager.add_expire_last_will(ExpireLastWill {
                client_id: will_message.client_id.clone(),
                delay_sec: now_second() + delay,
                cluster_name: req.cluster_name.to_owned(),
            });
        }
        Ok(None) => {}
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    }

    let data = StorageData::new(
        StorageDataType::MqttDeleteSession,
        DeleteSessionRequest::encode_to_vec(&req),
    );

    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };

    Ok(Response::new(DeleteSessionReply::default()))
}
