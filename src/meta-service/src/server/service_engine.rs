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

use crate::controller::call_broker::call::BrokerCallManager;
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::server::services::engine::segment::{
    create_segment_by_req, delete_segment_by_req, list_segment_by_req, list_segment_meta_by_req,
    seal_up_segment_req, update_start_time_by_segment_meta_by_req,
};
use crate::server::services::engine::shard::{
    create_shard_by_req, delete_shard_by_req, list_shard_by_req,
};
use grpc_clients::pool::ClientPool;
use prost_validate::Validator;
use protocol::meta::meta_service_journal::engine_service_server::EngineService;
use protocol::meta::meta_service_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    ListShardReply, ListShardRequest, SealUpSegmentReply, SealUpSegmentRequest,
    UpdateStartTimeBySegmentMetaReply, UpdateStartTimeBySegmentMetaRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcEngineService {
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    call_manager: Arc<BrokerCallManager>,
    client_pool: Arc<ClientPool>,
}

impl GrpcEngineService {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cache_manager: Arc<CacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        call_manager: Arc<BrokerCallManager>,
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

    fn validate_request<T: Validator>(&self, req: &T) -> Result<(), Status> {
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))
    }

    fn to_status(e: MetaServiceError) -> Status {
        let msg = e.to_string();
        match e {
            MetaServiceError::ShardDoesNotExist(_)
            | MetaServiceError::SegmentDoesNotExist(_)
            | MetaServiceError::SegmentMetaDoesNotExist(_)
            | MetaServiceError::NodeDoesNotExist(_)
            | MetaServiceError::UserDoesNotExist(_)
            | MetaServiceError::TopicDoesNotExist(_)
            | MetaServiceError::SessionDoesNotExist(_)
            | MetaServiceError::ConnectorNotFound(_)
            | MetaServiceError::SchemaDoesNotExist(_)
            | MetaServiceError::SubscribeDoesNotExist(_)
            | MetaServiceError::WillMessageDoesNotExist(_)
            | MetaServiceError::SchemaNotFound(_)
            | MetaServiceError::ClusterDoesNotExist(_) => Status::not_found(msg),

            MetaServiceError::TopicAlreadyExist(_)
            | MetaServiceError::ConnectorAlreadyExist(_)
            | MetaServiceError::UserAlreadyExist(_)
            | MetaServiceError::SchemaAlreadyExist(_) => Status::already_exists(msg),

            MetaServiceError::RequestParamsNotEmpty(_)
            | MetaServiceError::InvalidSegmentGreaterThan(_, _)
            | MetaServiceError::InvalidSegmentLessThan(_, _) => Status::invalid_argument(msg),

            MetaServiceError::NotEnoughEngineNodes(_, _)
            | MetaServiceError::ShardHasEnoughSegment(_)
            | MetaServiceError::NumberOfReplicasIsIncorrect(_, _)
            | MetaServiceError::NoAvailableBrokerNode
            | MetaServiceError::SegmentStateError(_, _, _)
            | MetaServiceError::NoAllowDeleteSegment(_, _)
            | MetaServiceError::SegmentWrongState(_) => Status::failed_precondition(msg),

            _ => Status::internal(msg),
        }
    }
}

#[tonic::async_trait]
impl EngineService for GrpcEngineService {
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

    async fn seal_up_segment(
        &self,
        request: Request<SealUpSegmentRequest>,
    ) -> Result<Response<SealUpSegmentReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        seal_up_segment_req(
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

    async fn update_start_time_by_segment_meta(
        &self,
        request: Request<UpdateStartTimeBySegmentMetaRequest>,
    ) -> Result<Response<UpdateStartTimeBySegmentMetaReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        update_start_time_by_segment_meta_by_req(
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
