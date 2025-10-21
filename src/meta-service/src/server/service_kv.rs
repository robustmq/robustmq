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

use crate::server::services::kv::{
    delete_by_req, exists_by_req, get_by_req, get_prefix_by_req, list_shard_by_req, set_by_req,
};
use protocol::meta::meta_service_kv::kv_service_server::KvService;
use protocol::meta::meta_service_kv::{
    DeleteReply, DeleteRequest, ExistsReply, ExistsRequest, GetPrefixReply, GetPrefixRequest,
    GetReply, GetRequest, ListShardReply, ListShardRequest, SetReply, SetRequest,
};
use tonic::{Request, Response, Status};

use crate::raft::route::apply::StorageDriver;
use rocksdb_engine::rocksdb::RocksDBEngine;

pub struct GrpcKvService {
    raft_machine_apply: Arc<StorageDriver>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcKvService {
    pub fn new(
        raft_machine_apply: Arc<StorageDriver>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcKvService {
            raft_machine_apply,
            rocksdb_engine_handler,
        }
    }
}

#[tonic::async_trait]
impl KvService for GrpcKvService {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetReply>, Status> {
        let req = request.into_inner();

        set_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteReply>, Status> {
        let req = request.into_inner();

        delete_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetReply>, Status> {
        let req = request.into_inner();

        get_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn exists(
        &self,
        request: Request<ExistsRequest>,
    ) -> Result<Response<ExistsReply>, Status> {
        let req = request.into_inner();

        exists_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_shard(
        &self,
        request: Request<ListShardRequest>,
    ) -> Result<Response<ListShardReply>, Status> {
        let req = request.into_inner();

        list_shard_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn get_prefix(
        &self,
        request: Request<GetPrefixRequest>,
    ) -> Result<Response<GetPrefixReply>, Status> {
        let req = request.into_inner();

        get_prefix_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }
}
