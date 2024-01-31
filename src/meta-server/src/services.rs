/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use super::cluster::Cluster;
use super::errors::MetaError;
use crate::raft::message::{RaftMessage, RaftResponseMesage};
use crate::storage::raft_core::RaftRocksDBStorageCore;
use crate::storage::schema::{StorageData, StorageDataStructBroker, StorageDataType};
use bincode::serialize;
use common::log::info_meta;
use prost::Message as _;
use protocol::robust::meta::{
    meta_service_server::MetaService, SendRaftMessageReply, SendRaftMessageRequest,
};
use protocol::robust::meta::{
    DeleteReply, DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, HeartbeatReply,
    HeartbeatRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest, SetReply, SetRequest,
};
use raft::eraftpb::{ConfChange, Message as raftPreludeMessage, MessageType};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{self, Receiver};
use tokio::time::timeout;

use std::sync::{Arc, RwLock};
use std::time::Duration;
use tonic::{Request, Response, Status};

pub struct GrpcService {
    cluster: Arc<RwLock<Cluster>>,
    raft_sender: Sender<RaftMessage>,
    rocksdb_storage: Arc<RwLock<RaftRocksDBStorageCore>>,
}

impl GrpcService {
    pub fn new(
        cluster: Arc<RwLock<Cluster>>,
        raft_sender: Sender<RaftMessage>,
        rocksdb_storage: Arc<RwLock<RaftRocksDBStorageCore>>,
    ) -> Self {
        GrpcService {
            cluster,
            raft_sender,
            rocksdb_storage,
        }
    }

    fn rewrite_leader(&self) -> bool {
        return self.cluster.read().unwrap().is_leader();
    }

    fn verify(&self) -> Result<(), MetaError> {
        let cluster = self.cluster.read().unwrap();

        if cluster.leader_alive() {
            return Err(MetaError::MetaClusterNotLeaderNode);
        }

        return Ok(());
    }

    async fn apply_raft_machine(&self, data: StorageData, action: String) -> Result<(), MetaError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();

        let _ = self
            .raft_sender
            .send(RaftMessage::Propose {
                data: serialize(&data).unwrap(),
                chan: sx,
            })
            .await;

        if !recv_chan_resp(rx).await {
            return Err(MetaError::MetaLogCommitTimeout(action));
        }
        return Ok(());
    }
}

async fn recv_chan_resp(rx: Receiver<RaftResponseMesage>) -> bool {
    let res = timeout(Duration::from_secs(30), async {
        match rx.await {
            Ok(val) => {
                return val;
            }
            Err(_) => {
                return RaftResponseMesage::Fail;
            }
        }
    });
    match res.await {
        Ok(_) => return true,
        Err(_) => {
            return false;
        }
    }
}

#[tonic::async_trait]
impl MetaService for GrpcService {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetReply>, Status> {

        if self.rewrite_leader() {
            let leader_addr = self.cluster.read().unwrap().leader_addr();
            match broker_register(&leader_addr, request.into_inner()).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        let node_id = request.into_inner().node_id;
        let addr = "127.0.0.1".to_string();

        let value = StorageDataStructBroker { node_id, addr };
        let data = StorageData::new(
            StorageDataType::RegisterBroker,
            serde_json::to_string(&value).unwrap(),
        );

        match self
            .apply_raft_machine(data, "broker_register".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(SetReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
        return Ok(Response::new(SetReply::default()));
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetReply>, Status> {
        return Ok(Response::new(GetReply::default()));
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteReply>, Status> {
        return Ok(Response::new(DeleteReply::default()));
    }

    async fn exists(
        &self,
        request: Request<ExistsRequest>,
    ) -> Result<Response<ExistsReply>, Status> {
        return Ok(Response::new(ExistsReply::default()));
    }
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatReply>, Status> {
        return Ok(Response::new(HeartbeatReply::default()));
    }

    async fn send_raft_message(
        &self,
        request: Request<SendRaftMessageRequest>,
    ) -> Result<Response<SendRaftMessageReply>, Status> {
        let message = raftPreludeMessage::decode(request.into_inner().message.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();

        match self
            .raft_sender
            .send(RaftMessage::Raft { message, chan: sx })
            .await
        {
            Ok(_) => {
                if !recv_chan_resp(rx).await {
                    return Err(Status::cancelled(
                        MetaError::MetaLogCommitTimeout("send_raft_message".to_string())
                            .to_string(),
                    ));
                }
            }
            Err(e) => {
                return Err(Status::aborted(
                    MetaError::RaftStepCommitFail(e.to_string()).to_string(),
                ));
            }
        }
        Ok(Response::new(SendRaftMessageReply::default()))
    }

    async fn send_raft_conf_change(
        &self,
        request: Request<SendRaftConfChangeRequest>,
    ) -> Result<Response<SendRaftConfChangeReply>, Status> {
        let change = ConfChange::decode(request.into_inner().message.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        info_meta(&format!(
            "grpc receive send_raft_conf_change change data:{:?}",
            change
        ));
        match self
            .raft_sender
            .send(RaftMessage::ConfChange { change, chan: sx })
            .await
        {
            Ok(_) => {
                if !recv_chan_resp(rx).await {
                    return Err(Status::cancelled(
                        MetaError::MetaLogCommitTimeout("send_raft_conf_change".to_string())
                            .to_string(),
                    ));
                }
            }
            Err(e) => {
                return Err(Status::aborted(
                    MetaError::RaftStepCommitFail(e.to_string()).to_string(),
                ));
            }
        }
        Ok(Response::new(SendRaftConfChangeReply::default()))
    }
}
