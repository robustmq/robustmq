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


use common_base::error::common::CommonError;
use mobc::Manager;
use protocol::placement_center::placement_center_openraft::open_raft_service_client::OpenRaftServiceClient;
use protocol::placement_center::placement_center_openraft::{
    AddLearnerReply, AddLearnerRequest, AppendReply, AppendRequest, ChangeMembershipReply,
    ChangeMembershipRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};
use tonic::transport::Channel;

use crate::pool::ClientPool;

pub mod call;

#[derive(Debug, Clone)]
pub enum OpenRaftServiceRequest {
    Vote(VoteRequest),
    Append(AppendRequest),
    Snapshot(SnapshotRequest),
    AddLearner(AddLearnerRequest),
    ChangeMembership(ChangeMembershipRequest),
}

#[derive(Debug, Clone)]
pub enum OpenRaftServiceReply {
    Vote(VoteReply),
    Append(AppendReply),
    Snapshot(SnapshotReply),
    AddLearner(AddLearnerReply),
    ChangeMembership(ChangeMembershipReply),
}

pub(super) async fn call_open_raft_service_once(
    client_pool: &ClientPool,
    addr: &str,
    request: OpenRaftServiceRequest,
) -> Result<OpenRaftServiceReply, CommonError> {
    use OpenRaftServiceRequest::*;

    match request {
        Vote(request) => {
            let mut client = client_pool.placement_center_openraft_services_client(addr).await?;
            let reply = client.vote(request).await?;
            Ok(OpenRaftServiceReply::Vote(reply.into_inner()))
        }
        Append(request) => {
            let mut client = client_pool.placement_center_openraft_services_client(addr).await?;
            let reply = client.append(request).await?;
            Ok(OpenRaftServiceReply::Append(reply.into_inner()))
        }
        Snapshot(request) => {
            let mut client = client_pool.placement_center_openraft_services_client(addr).await?;
            let reply = client.snapshot(request).await?;
            Ok(OpenRaftServiceReply::Snapshot(reply.into_inner()))
        }
        AddLearner(request) => {
            let mut client = client_pool.placement_center_openraft_services_client(addr).await?;
            let reply = client.add_learner(request).await?;
            Ok(OpenRaftServiceReply::AddLearner(reply.into_inner()))
        }
        ChangeMembership(request) => {
            let mut client = client_pool.placement_center_openraft_services_client(addr).await?;
            let reply = client.change_membership(request).await?;
            Ok(OpenRaftServiceReply::ChangeMembership(reply.into_inner()))
        }
    }
}

#[derive(Clone)]
pub struct OpenRaftServiceManager {
    pub addr: String,
}

impl OpenRaftServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for OpenRaftServiceManager {
    type Connection = OpenRaftServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let addr = format!("http://{}", self.addr.clone());

        match OpenRaftServiceClient::connect(addr.clone()).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => return Err(CommonError::CommonError(format!("{},{}", err, addr))),
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}
