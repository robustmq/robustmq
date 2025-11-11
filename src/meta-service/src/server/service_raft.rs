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

use crate::raft::services::{
    add_learner_by_req, append_by_req, change_membership_by_req, snapshot_by_req, vote_by_req,
};
use crate::raft::type_config::TypeConfig;
use openraft::Raft;
use protocol::meta::meta_service_openraft::open_raft_service_server::OpenRaftService;
use protocol::meta::meta_service_openraft::{
    AddLearnerReply, AddLearnerRequest, AppendReply, AppendRequest, ChangeMembershipReply,
    ChangeMembershipRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};
use tonic::{Request, Response, Status};

pub struct GrpcOpenRaftServices {
    raft_node: Raft<TypeConfig>,
}

impl GrpcOpenRaftServices {
    pub fn new(raft_node: Raft<TypeConfig>) -> Self {
        GrpcOpenRaftServices { raft_node }
    }
}

#[tonic::async_trait]
impl OpenRaftService for GrpcOpenRaftServices {
    async fn vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteReply>, Status> {
        let req = request.into_inner();
        vote_by_req(&self.raft_node, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn append(
        &self,
        request: Request<AppendRequest>,
    ) -> Result<Response<AppendReply>, Status> {
        let req = request.into_inner();
        append_by_req(&self.raft_node, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn snapshot(
        &self,
        request: Request<SnapshotRequest>,
    ) -> Result<Response<SnapshotReply>, Status> {
        let req = request.into_inner();
        snapshot_by_req(&self.raft_node, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn add_learner(
        &self,
        request: Request<AddLearnerRequest>,
    ) -> Result<Response<AddLearnerReply>, Status> {
        let req = request.into_inner();
        add_learner_by_req(&self.raft_node, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn change_membership(
        &self,
        request: Request<ChangeMembershipRequest>,
    ) -> Result<Response<ChangeMembershipReply>, Status> {
        let req = request.into_inner();
        change_membership_by_req(&self.raft_node, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }
}
