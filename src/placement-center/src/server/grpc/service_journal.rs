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
use std::sync::Arc;

use prost::Message;
use protocol::placement_center::placement_center_journal::engine_service_server::EngineService;
use protocol::placement_center::placement_center_journal::{
    CreateSegmentReply, CreateSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest, GetShardReply,
    GetShardRequest,
};
use tonic::{Request, Response, Status};

use crate::storage::route::apply::RaftMachineApply;
use crate::storage::route::data::{StorageData, StorageDataType};

pub struct GrpcEngineService {
    raft_machine_apply: Arc<RaftMachineApply>,
}

impl GrpcEngineService {
    pub fn new(raft_machine_apply: Arc<RaftMachineApply>) -> Self {
        GrpcEngineService { raft_machine_apply }
    }
}

#[tonic::async_trait]
impl EngineService for GrpcEngineService {
    async fn create_shard(
        &self,
        request: Request<CreateShardRequest>,
    ) -> Result<Response<CreateShardReply>, Status> {
        let req = request.into_inner();

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalCreateShard,
            CreateShardRequest::encode_to_vec(&req),
        );
        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CreateShardReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_shard(
        &self,
        request: Request<DeleteShardRequest>,
    ) -> Result<Response<DeleteShardReply>, Status> {
        let req = request.into_inner();

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalDeleteShard,
            DeleteShardRequest::encode_to_vec(&req),
        );
        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(DeleteShardReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get_shard(
        &self,
        _: Request<GetShardRequest>,
    ) -> Result<Response<GetShardReply>, Status> {
        let result = GetShardReply::default();

        return Ok(Response::new(result));
    }

    async fn create_segment(
        &self,
        request: Request<CreateSegmentRequest>,
    ) -> Result<Response<CreateSegmentReply>, Status> {
        let req = request.into_inner();

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalCreateSegment,
            CreateSegmentRequest::encode_to_vec(&req),
        );
        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CreateSegmentReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_segment(
        &self,
        request: Request<DeleteSegmentRequest>,
    ) -> Result<Response<DeleteSegmentReply>, Status> {
        let req = request.into_inner();

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalDeleteSegment,
            DeleteSegmentRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(DeleteSegmentReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
