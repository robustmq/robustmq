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
use crate::server::services::nats::subscribe::{
    create_nats_subscribe_by_req, delete_nats_subscribe_by_req, list_nats_subscribe_by_req,
};
use node_call::NodeCallManager;
use prost_validate::Validator;
use protocol::meta::meta_service_nats::nats_service_server::NatsService;
use protocol::meta::meta_service_nats::{
    CreateNatsSubscribeReply, CreateNatsSubscribeRequest, DeleteNatsSubscribeReply,
    DeleteNatsSubscribeRequest, ListNatsSubscribeReply, ListNatsSubscribeRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::{Request, Response, Status};

pub struct GrpcNatsService {
    raft_manager: Arc<MultiRaftManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    call_manager: Arc<NodeCallManager>,
}

impl GrpcNatsService {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        call_manager: Arc<NodeCallManager>,
    ) -> Self {
        GrpcNatsService {
            raft_manager,
            rocksdb_engine_handler,
            call_manager,
        }
    }

    fn validate_request<T: Validator>(&self, req: &T) -> Result<(), Status> {
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))
    }

    fn to_status<E: ToString>(e: E) -> Status {
        Status::internal(e.to_string())
    }
}

#[tonic::async_trait]
impl NatsService for GrpcNatsService {
    type ListNatsSubscribeStream =
        Pin<Box<dyn Stream<Item = Result<ListNatsSubscribeReply, Status>> + Send>>;

    async fn create_nats_subscribe(
        &self,
        request: Request<CreateNatsSubscribeRequest>,
    ) -> Result<Response<CreateNatsSubscribeReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;
        create_nats_subscribe_by_req(
            &self.raft_manager,
            &self.call_manager,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_nats_subscribe(
        &self,
        request: Request<DeleteNatsSubscribeRequest>,
    ) -> Result<Response<DeleteNatsSubscribeReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;
        delete_nats_subscribe_by_req(
            &self.raft_manager,
            &self.call_manager,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn list_nats_subscribe(
        &self,
        request: Request<ListNatsSubscribeRequest>,
    ) -> Result<Response<Self::ListNatsSubscribeStream>, Status> {
        let req = request.into_inner();
        list_nats_subscribe_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }
}
