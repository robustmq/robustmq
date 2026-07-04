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

use crate::raft::manager::MultiRaftManager;
use protocol::meta::meta_service_kafka::kafka_service_server::KafkaService;
use protocol::meta::meta_service_kafka::{GetCoordinatorLeaderReply, GetCoordinatorLeaderRequest};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcKafkaService {
    raft_manager: Arc<MultiRaftManager>,
}

impl GrpcKafkaService {
    pub fn new(raft_manager: Arc<MultiRaftManager>) -> Self {
        GrpcKafkaService { raft_manager }
    }
}

#[tonic::async_trait]
impl KafkaService for GrpcKafkaService {
    async fn get_coordinator_leader(
        &self,
        _request: Request<GetCoordinatorLeaderRequest>,
    ) -> Result<Response<GetCoordinatorLeaderReply>, Status> {
        // For now the Kafka group coordinator is simply the metadata-raft leader.
        let reply = match self.raft_manager.metadata_leader() {
            Some(leader_node_id) => GetCoordinatorLeaderReply {
                leader_node_id,
                has_leader: true,
            },
            None => GetCoordinatorLeaderReply {
                leader_node_id: 0,
                has_leader: false,
            },
        };
        Ok(Response::new(reply))
    }
}
