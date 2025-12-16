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

use crate::core::cache::CacheManager;
use crate::inner::services::{
    delete_segment_file_by_req, delete_shard_file_by_req, get_segment_delete_status_by_req,
    get_shard_delete_status_by_req, update_cache_by_req,
};
use crate::segment::manager::SegmentFileManager;
use protocol::journal::journal_inner::journal_server_inner_service_server::JournalServerInnerService;
use protocol::journal::journal_inner::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcJournalServerInnerService {
    cache_manager: Arc<CacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcJournalServerInnerService {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        segment_file_manager: Arc<SegmentFileManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcJournalServerInnerService {
            cache_manager,
            segment_file_manager,
            rocksdb_engine_handler,
        }
    }
}

#[tonic::async_trait]
impl JournalServerInnerService for GrpcJournalServerInnerService {
    async fn update_cache(
        &self,
        request: Request<UpdateJournalCacheRequest>,
    ) -> Result<Response<UpdateJournalCacheReply>, Status> {
        let request = request.into_inner();
        update_cache_by_req(&self.cache_manager, &self.segment_file_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

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
