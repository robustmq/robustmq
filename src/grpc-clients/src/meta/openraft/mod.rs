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
use protocol::meta::placement_center_openraft::open_raft_service_client::OpenRaftServiceClient;
use protocol::meta::placement_center_openraft::{
    AddLearnerReply, AddLearnerRequest, AppendReply, AppendRequest, ChangeMembershipReply,
    ChangeMembershipRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

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
            Err(err) => return Err(CommonError::CommonError(format!("{err},{addr}"))),
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

impl_retriable_request!(
    VoteRequest,
    OpenRaftServiceClient<Channel>,
    VoteReply,
    placement_center_openraft_services_client,
    vote,
    true
);

impl_retriable_request!(
    AppendRequest,
    OpenRaftServiceClient<Channel>,
    AppendReply,
    placement_center_openraft_services_client,
    append,
    true
);

impl_retriable_request!(
    SnapshotRequest,
    OpenRaftServiceClient<Channel>,
    SnapshotReply,
    placement_center_openraft_services_client,
    snapshot,
    true
);

impl_retriable_request!(
    AddLearnerRequest,
    OpenRaftServiceClient<Channel>,
    AddLearnerReply,
    placement_center_openraft_services_client,
    add_learner,
    true
);

impl_retriable_request!(
    ChangeMembershipRequest,
    OpenRaftServiceClient<Channel>,
    ChangeMembershipReply,
    placement_center_openraft_services_client,
    change_membership,
    true
);
