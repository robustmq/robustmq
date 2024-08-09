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
use crate::raft::apply::{RaftMachineApply, StorageData, StorageDataType};
use crate::raft::metadata::RaftGroupMetadata;
use clients::{
    placement::journal::call::{create_segment, create_shard, delete_segment, delete_shard},
    poll::ClientPool,
};
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    journal::{
        engine_service_server::EngineService, CreateSegmentRequest, CreateShardRequest,
        DeleteSegmentRequest, DeleteShardRequest, GetShardReply, GetShardRequest,
    },
};
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};

pub struct GrpcEngineService {
    placement_center_storage: Arc<RaftMachineApply>,
    placement_cache: Arc<RwLock<RaftGroupMetadata>>,
    client_poll: Arc<ClientPool>,
}

impl GrpcEngineService {
    pub fn new(
        placement_center_storage: Arc<RaftMachineApply>,
        placement_cache: Arc<RwLock<RaftGroupMetadata>>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        GrpcEngineService {
            placement_center_storage,
            placement_cache,
            client_poll,
        }
    }

    fn rewrite_leader(&self) -> bool {
        return !self.placement_cache.read().unwrap().is_leader();
    }
}

#[tonic::async_trait]
impl EngineService for GrpcEngineService {
    async fn create_shard(
        &self,
        request: Request<CreateShardRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match create_shard(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalCreateShard,
            CreateShardRequest::encode_to_vec(&req),
        );
        match self
            .placement_center_storage
            .apply_propose_message(data, "create_shard".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_shard(
        &self,
        request: Request<DeleteShardRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match delete_shard(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalDeleteShard,
            DeleteShardRequest::encode_to_vec(&req),
        );
        match self
            .placement_center_storage
            .apply_propose_message(data, "delete_shard".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get_shard(
        &self,
        request: Request<GetShardRequest>,
    ) -> Result<Response<GetShardReply>, Status> {
        // let req = request.into_inner();
        // let shard_info = self
        //     .cluster_storage
        //     .get_shard(req.cluster_name.clone(), req.shard_name);
        let result = GetShardReply::default();
        // if shard_info.is_none() {
        //     let si = shard_info.unwrap();
        //     result.cluster_name = req.cluster_name;
        //     result.shard_id = si.shard_id;
        //     result.shard_name = si.shard_name;
        //     result.replica = si.replica;
        //     result.replicas = serialize(&si.replicas).unwrap();
        // }

        return Ok(Response::new(result));
    }

    async fn create_segment(
        &self,
        request: Request<CreateSegmentRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match create_segment(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalCreateSegment,
            CreateSegmentRequest::encode_to_vec(&req),
        );
        match self
            .placement_center_storage
            .apply_propose_message(data, "create_segment".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_segment(
        &self,
        request: Request<DeleteSegmentRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match delete_segment(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalDeleteSegment,
            DeleteSegmentRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "delete_segment".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
