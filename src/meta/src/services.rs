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

use super::errors::MetaError;
use crate::raft::{
    message::{RaftMessage, RaftResponseMesage},
    node::{Node, NodeRaftState},
};
use common::log::{debug, info};
use protocol::robust::meta::{
    meta_service_server::MetaService, BrokerRegisterReply, BrokerRegisterRequest,
    BrokerUnRegisterReply, BrokerUnRegisterRequest, FindLeaderReply, FindLeaderRequest,
    HeartbeatReply, HeartbeatRequest, TransformLeaderReply, TransformLeaderRequest, VoteReply,
    VoteRequest,
};
use std::sync::{Arc, RwLock};
use tokio::sync::{mpsc::Sender, oneshot};
use tonic::{Request, Response, Status};

pub struct GrpcService {
    node: Arc<RwLock<Node>>,
    raft_sender: Sender<RaftMessage>,
}

impl GrpcService {
    pub fn new(node: Arc<RwLock<Node>>, raft_sender: Sender<RaftMessage>) -> Self {
        GrpcService {
            node: node,
            raft_sender,
        }
    }
}

#[tonic::async_trait]
impl MetaService for GrpcService {
    async fn find_leader(
        &self,
        _: Request<FindLeaderRequest>,
    ) -> Result<Response<FindLeaderReply>, Status> {
        let node = self.node.read().unwrap();
        let mut reply = FindLeaderReply::default();

        // If the Leader exists in the cluster, the current Leader information is displayed
        if node.raft_state == NodeRaftState::Leader {
            reply.leader_id = node.leader_id.clone().unwrap();
            reply.leader_ip = node.leader_ip.clone().unwrap();
            return Ok(Response::new(reply));
        }
        Ok(Response::new(reply))
    }

    async fn vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteReply>, Status> {
        let mut node = self.node.write().unwrap();

        if node.raft_state == NodeRaftState::Leader {
            return Err(Status::already_exists(
                MetaError::LeaderExistsNotAllowElection.to_string(),
            ));
        }

        if let Some(voter) = node.voter {
            return Err(Status::already_exists(
                MetaError::NodeBeingVotedOn { node_id: voter }.to_string(),
            ));
        }

        let req_node_id = request.into_inner().node_id;

        if req_node_id <= 0 {
            return Err(Status::already_exists(
                MetaError::UnavailableNodeId {
                    node_id: req_node_id,
                }
                .to_string(),
            ));
        }

        node.voter = Some(req_node_id);

        Ok(Response::new(VoteReply {
            vote_node_id: req_node_id,
        }))
    }

    async fn transform_leader(
        &self,
        request: Request<TransformLeaderRequest>,
    ) -> Result<Response<TransformLeaderReply>, Status> {
        let req = request.into_inner();
        let mut node = self.node.write().unwrap();

        if node.raft_state == NodeRaftState::Leader {
            return Err(Status::already_exists(
                MetaError::LeaderExistsNotAllowElection.to_string(),
            ));
        }
        node.voter = None;
        node.leader_id = Some(req.node_id);
        node.leader_ip = Some(req.node_ip);
        node.raft_state = NodeRaftState::Follower;

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
        let node_id = request.into_inner().node_id;
        info(&format!(
            "Register Broker node information, node ID {}",
            node_id
        ));
        Ok(Response::new(BrokerRegisterReply::default()))
    }

    async fn broker_un_register(
        &self,
        request: Request<BrokerUnRegisterRequest>,
    ) -> Result<Response<BrokerUnRegisterReply>, Status> {
        let node_id = request.into_inner().node_id;
        info(&format!("Broker node is stopped, node ID {}", node_id));
        let (sx, mut rx) = oneshot::channel::<RaftResponseMesage>();
        let _ = self
            .raft_sender
            .send(RaftMessage::Propose {
                data: "sdafasdfas".to_string().into_bytes(),
                chan: sx,
            })
            .await;
        // loop {
        //     match rx.try_recv() {
        //         Ok(_) =>break,
        //         Err(err) => {
        //             return Err(Status::already_exists(err.to_string()));
        //         }
        //     }
        // }
        Ok(Response::new(BrokerUnRegisterReply::default()))
    }
}
