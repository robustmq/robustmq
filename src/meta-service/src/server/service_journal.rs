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

use crate::controller::storage::call_node::JournalInnerCallManager;
use crate::core::cache::CacheManager;
use crate::raft::manager::MultiRaftManager;
use crate::server::services::journal::segment::{
    create_segment_by_req, delete_segment_by_req, list_segment_by_req, list_segment_meta_by_req,
    update_segment_meta_by_req, update_segment_status_req,
};
use crate::server::services::journal::shard::{
    create_shard_by_req, delete_shard_by_req, list_shard_by_req,
};
use grpc_clients::pool::ClientPool;
use prost_validate::Validator;
use protocol::meta::meta_service_journal::engine_service_server::EngineService;
use protocol::meta::meta_service_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    ListShardReply, ListShardRequest, UpdateSegmentMetaReply, UpdateSegmentMetaRequest,
    UpdateSegmentStatusReply, UpdateSegmentStatusRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcEngineService {
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    call_manager: Arc<JournalInnerCallManager>,
    client_pool: Arc<ClientPool>,
}

impl GrpcEngineService {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cache_manager: Arc<CacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        call_manager: Arc<JournalInnerCallManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        GrpcEngineService {
            raft_manager,
            cache_manager,
            rocksdb_engine_handler,
            call_manager,
            client_pool,
        }
    }

    // Helper: Validate request and convert errors
    fn validate_request<T: Validator>(&self, req: &T) -> Result<(), Status> {
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))
    }

    // Helper: Convert MetaServiceError to Status
    fn to_status<E: ToString>(e: E) -> Status {
        Status::internal(e.to_string())
    }
}

#[tonic::async_trait]
impl EngineService for GrpcEngineService {
    // Shard
    async fn list_shard(
        &self,
        request: Request<ListShardRequest>,
    ) -> Result<Response<ListShardReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_shard_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_shard(
        &self,
        request: Request<CreateShardRequest>,
    ) -> Result<Response<CreateShardReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_shard_by_req(
            &self.cache_manager,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_shard(
        &self,
        request: Request<DeleteShardRequest>,
    ) -> Result<Response<DeleteShardReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_shard_by_req(
            &self.raft_manager,
            &self.cache_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    // Segment
    async fn list_segment(
        &self,
        request: Request<ListSegmentRequest>,
    ) -> Result<Response<ListSegmentReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_segment_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_next_segment(
        &self,
        request: Request<CreateNextSegmentRequest>,
    ) -> Result<Response<CreateNextSegmentReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_segment_by_req(
            &self.cache_manager,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_segment(
        &self,
        request: Request<DeleteSegmentRequest>,
    ) -> Result<Response<DeleteSegmentReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_segment_by_req(
            &self.cache_manager,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn update_segment_status(
        &self,
        request: Request<UpdateSegmentStatusRequest>,
    ) -> Result<Response<UpdateSegmentStatusReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        update_segment_status_req(
            &self.cache_manager,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    // Segment Metadata
    async fn list_segment_meta(
        &self,
        request: Request<ListSegmentMetaRequest>,
    ) -> Result<Response<ListSegmentMetaReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_segment_meta_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn update_segment_meta(
        &self,
        request: Request<UpdateSegmentMetaRequest>,
    ) -> Result<Response<UpdateSegmentMetaReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        update_segment_meta_by_req(
            &self.cache_manager,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }
}
