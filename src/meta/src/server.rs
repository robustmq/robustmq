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

use super::{
    errors::MetaError,
    node::{Node, NodeRaftState},
    proto::meta::{
        meta_service_server::MetaService, FindLeaderReply, FindLeaderRequest,
        TransformLeaderReply, TransformLeaderRequest, VoteReply, VoteRequest, HeartbeatRequest, HeartbeatReply,
    },
};
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};

pub struct GrpcService {
    node: Arc<RwLock<Node>>,
}

impl GrpcService {
    pub fn new(node: Arc<RwLock<Node>>) -> Self {
        GrpcService { node: node }
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
        
        Ok(Response::new(HeartbeatReply::default()))
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use tokio::runtime::Runtime;
    use tonic_build;

    use crate::proto::meta::{meta_service_client::MetaServiceClient, FindLeaderRequest};

    #[test]
    fn create_rust_pb() {
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/meta/proto") // you can change the generated code's location
            .compile(
                &["src/meta/proto/meta.proto"],
                &["src/meta/proto/"], // specify the root location to search proto dependencies
            )
            .unwrap();
    }

    #[test]
    fn grpc_client() {
        let runtime: Runtime = tokio::runtime::Builder::new_multi_thread()
            // .worker_threads(self.config.work_thread.unwrap() as usize)
            .max_blocking_threads(2048)
            .thread_name("meta-http")
            .enable_io()
            .build()
            .unwrap();

        let _gurad = runtime.enter();

        runtime.spawn(async move {
            let mut client = MetaServiceClient::connect("http://127.0.0.1:1228")
                .await
                .unwrap();

            let request = tonic::Request::new(FindLeaderRequest {});
            let response = client.find_leader(request).await.unwrap();

            println!("response={:?}", response);
        });

        loop {
            thread::sleep(Duration::from_secs(300));
        }
    }
}
