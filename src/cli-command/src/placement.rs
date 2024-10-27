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

use std::sync::Arc;

use grpc_clients::placement::openraft::call::{
    placement_openraft_add_learner, placement_openraft_change_membership,
};
use grpc_clients::placement::placement::call::cluster_status;
use grpc_clients::pool::ClientPool;
use protocol::placement_center::placement_center_inner::ClusterStatusRequest;
use protocol::placement_center::placement_center_openraft::{
    AddLearnerRequest, ChangeMembershipRequest, Node,
};

use crate::{error_info, grpc_addr};

#[derive(Clone)]
pub struct PlacementCliCommandParam {
    pub server: String,
    pub action: PlacementActionType,
}

#[derive(Clone)]
pub enum PlacementActionType {
    Status,
    AddLearner(AddLearnerCliRequset),
    ChangeMembership(ChangeMembershipCliRequest),
}

#[derive(Clone)]
pub struct AddLearnerCliRequset {
    node_id: u64,
    rpc_addr: String,
    blocking: bool,
}

impl AddLearnerCliRequset {
    pub fn new(node_id: u64, rpc_addr: String, blocking: bool) -> Self {
        AddLearnerCliRequset {
            node_id,
            rpc_addr,
            blocking,
        }
    }
}

#[derive(Clone)]
pub struct ChangeMembershipCliRequest {
    members: Vec<u64>,
    retain: bool,
}

impl ChangeMembershipCliRequest {
    pub fn new(members: Vec<u64>, retain: bool) -> Self {
        ChangeMembershipCliRequest { members, retain }
    }
}

pub struct PlacementCenterCommand {}

impl Default for PlacementCenterCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PlacementCenterCommand {
    pub fn new() -> Self {
        PlacementCenterCommand {}
    }
    pub async fn start(&self, params: PlacementCliCommandParam) {
        let client_poll = Arc::new(ClientPool::new(100));

        match params.action {
            PlacementActionType::Status => {
                self.status(client_poll.clone(), params).await;
            }
            PlacementActionType::AddLearner(ref request) => {
                self.add_learner(client_poll.clone(), params.clone(), request.clone())
                    .await;
            }
            PlacementActionType::ChangeMembership(ref request) => {
                self.change_membership(client_poll.clone(), params.clone(), request.clone())
                    .await;
            }
        }
    }

    async fn status(&self, client_poll: Arc<ClientPool>, params: PlacementCliCommandParam) {
        let request = ClusterStatusRequest {};
        match cluster_status(client_poll, grpc_addr(params.server), request).await {
            Ok(reply) => {
                println!("{}", reply.content);
            }
            Err(e) => {
                println!("Placement center cluster normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn add_learner(
        &self,
        client_poll: Arc<ClientPool>,
        params: PlacementCliCommandParam,
        cli_request: AddLearnerCliRequset,
    ) {
        let request = AddLearnerRequest {
            node_id: cli_request.node_id,
            node: Some(Node {
                node_id: cli_request.node_id,
                rpc_addr: cli_request.rpc_addr,
            }),
            blocking: cli_request.blocking,
        };

        match placement_openraft_add_learner(client_poll, grpc_addr(params.server), request).await {
            Ok(reply) => {
                println!("{:?}", reply.value);
            }
            Err(e) => {
                println!("Placement center add leaner normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn change_membership(
        &self,
        client_poll: Arc<ClientPool>,
        params: PlacementCliCommandParam,
        cli_request: ChangeMembershipCliRequest,
    ) {
        let request = ChangeMembershipRequest {
            members: cli_request.members,
            retain: cli_request.retain,
        };

        match placement_openraft_change_membership(client_poll, grpc_addr(params.server), request)
            .await
        {
            Ok(reply) => {
                println!("{:?}", reply.value);
            }
            Err(e) => {
                println!("Placement center cluster normal exception");
                error_info(e.to_string());
            }
        }
    }
}
