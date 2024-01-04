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
use crate::storage::schema::{StorageDataStructBroker, StorageData, StorageDataType};
use bincode::serialize;
use common::log::debug;
use prost::Message as _;
use protocol::robust::meta::{
    meta_service_server::MetaService, BrokerRegisterReply, BrokerRegisterRequest,
    BrokerUnRegisterReply, BrokerUnRegisterRequest, FindLeaderReply, FindLeaderRequest,
    HeartbeatReply, HeartbeatRequest, SendRaftMessageReply, SendRaftMessageRequest,
    TransformLeaderReply, TransformLeaderRequest, VoteReply, VoteRequest,
};
use raft::eraftpb::Message as raftPreludeMessage;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{self, Receiver};

use std::{
    sync::{Arc, RwLock},
    time::Instant,
};
use tonic::{Request, Response, Status};

pub struct GrpcService {
    cluster: Arc<RwLock<Cluster>>,
    raft_sender: Sender<RaftMessage>,
}

impl GrpcService {
    pub fn new(cluster: Arc<RwLock<Cluster>>, raft_sender: Sender<RaftMessage>) -> Self {
        GrpcService {
            cluster,
            raft_sender,
        }
    }

    pub fn wait_raft_commit(&self, mut rx: Receiver<RaftResponseMesage>) -> bool {
        let now = Instant::now();
        loop {
            match rx.try_recv() {
                Ok(_) => {
                    return true;
                }
                Err(_) => {
                    if now.elapsed().as_secs() > 30 {
                        return false;
                    }
                }
            }
        }
    }
}

#[tonic::async_trait]
impl MetaService for GrpcService {
    async fn find_leader(
        &self,
        _: Request<FindLeaderRequest>,
    ) -> Result<Response<FindLeaderReply>, Status> {
        let cluster = self.cluster.read().unwrap();
        let mut reply = FindLeaderReply::default();

        // If the Leader exists in the cluster, the current Leader information is displayed
        if cluster.is_leader() {
            if let Some(n) = cluster.leader.clone() {
                reply.leader_id = n.id;
                reply.leader_ip = n.ip;
                return Ok(Response::new(reply));
            }
        }
        Ok(Response::new(reply))
    }

    async fn vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteReply>, Status> {
        let cluster = self.cluster.read().unwrap();

        if cluster.is_leader() {
            return Err(Status::aborted(
                MetaError::LeaderExistsNotAllowElection.to_string(),
            ));
        }

        // if let Some(voter) = node.voter {
        //     return Err(Status::aborted(
        //         MetaError::NodeBeingVotedOn { node_id: voter }.to_string(),
        //     ));
        // }

        let req_node_id = request.into_inner().node_id;

        if req_node_id <= 0 {
            return Err(Status::aborted(
                MetaError::UnavailableNodeId {
                    node_id: req_node_id,
                }
                .to_string(),
            ));
        }

        // node.voter = Some(req_node_id);

        Ok(Response::new(VoteReply {
            vote_node_id: req_node_id,
        }))
    }

    async fn transform_leader(
        &self,
        request: Request<TransformLeaderRequest>,
    ) -> Result<Response<TransformLeaderReply>, Status> {
        let _ = request.into_inner();

        let reply = TransformLeaderReply::default();
        Ok(Response::new(reply))
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatReply>, Status> {
        let node_id = request.into_inner().node_id;
        debug(&format!("Receiving the message from node ID {}", node_id));
        Ok(Response::new(HeartbeatReply::default()))
    }

    async fn broker_register(
        &self,
        request: Request<BrokerRegisterRequest>,
    ) -> Result<Response<BrokerRegisterReply>, Status> {
        let id = request.into_inner().node_id;
        
        Ok(Response::new(BrokerRegisterReply::default()))
    }

    async fn broker_un_register(
        &self,
        request: Request<BrokerUnRegisterRequest>,
    ) -> Result<Response<BrokerUnRegisterReply>, Status> {
        let node_id = request.into_inner().node_id;
        let addr = "127.0.0.1".to_string();
        let value = StorageDataStructBroker { node_id, addr };

        let data = StorageData::new(
            StorageDataType::UnRegisterBroker,
            serialize(&value).unwrap(),
        );
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();

        let _ = self
            .raft_sender
            .send(RaftMessage::Propose {
                data: serialize(&data).unwrap(),
                chan: sx,
            })
            .await;

        if !self.wait_raft_commit(rx) {
            return Err(Status::cancelled(
                MetaError::MetaLogCommitTimeout("broker_un_register".to_string()).to_string(),
            ));
        }

        Ok(Response::new(BrokerUnRegisterReply::default()))
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
                if !self.wait_raft_commit(rx) {
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
}
