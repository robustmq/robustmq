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

use crate::core::cache::StorageCacheManager;
use crate::core::services::{
    delete_segment_file_by_req, delete_shard_file_by_req, get_segment_delete_status_by_req,
    get_shard_delete_status_by_req,
};
use crate::segment::storage::manager::SegmentFileManager;
use protocol::broker::broker_storage::broker_storage_service_server::BrokerStorageService;
use protocol::broker::broker_storage::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcBrokerStorageServerService {
    cache_manager: Arc<StorageCacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcBrokerStorageServerService {
    pub fn new(
        cache_manager: Arc<StorageCacheManager>,
        segment_file_manager: Arc<SegmentFileManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcBrokerStorageServerService {
            cache_manager,
            segment_file_manager,
            rocksdb_engine_handler,
        }
    }
}

#[tonic::async_trait]
impl BrokerStorageService for GrpcBrokerStorageServerService {
    async fn delete_shard_file(
        &self,
        request: Request<DeleteShardFileRequest>,
    ) -> Result<Response<DeleteShardFileReply>, Status> {
        let request = request.into_inner();
        delete_shard_file_by_req(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            &self.segment_file_manager,
            &request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn get_shard_delete_status(
        &self,
        request: Request<GetShardDeleteStatusRequest>,
    ) -> Result<Response<GetShardDeleteStatusReply>, Status> {
        let request = request.into_inner();
        get_shard_delete_status_by_req(&request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_segment_file(
        &self,
        request: Request<DeleteSegmentFileRequest>,
    ) -> Result<Response<DeleteSegmentFileReply>, Status> {
        let request = request.into_inner();
        delete_segment_file_by_req(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            &self.segment_file_manager,
            &request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn get_segment_delete_status(
        &self,
        request: Request<GetSegmentDeleteStatusRequest>,
    ) -> Result<Response<GetSegmentDeleteStatusReply>, Status> {
        let request = request.into_inner();
        get_segment_delete_status_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }
}
