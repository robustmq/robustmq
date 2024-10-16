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

use grpc_clients::placement::placement::call::cluster_status;
use grpc_clients::poll::ClientPool;
use protocol::placement_center::placement_center_inner::ClusterStatusRequest;

use crate::{error_info, grpc_addr};

#[derive(Clone)]
pub struct PlacementCliCommandParam {
    pub server: String,
    pub action: String,
}

pub enum PlacementActionType {
    STATUS,
}

impl From<String> for PlacementActionType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "status" => PlacementActionType::STATUS,
            _ => panic!("Invalid action type {}", s),
        }
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
        let action_type = PlacementActionType::from(params.action.clone());
        let client_poll = Arc::new(ClientPool::new(100));
        match action_type {
            PlacementActionType::STATUS => {
                self.status(client_poll.clone(), params.clone()).await;
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
}
