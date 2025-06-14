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

use common_config::journal::config::journal_server_conf;
use protocol::journal_server::journal_inner::journal_server_inner_service_server::JournalServerInnerService;
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};
use rocksdb_engine::RocksDBEngine;
use tonic::{Request, Response, Status};

use crate::core::cache::CacheManager;
use crate::core::notification::parse_notification;
use crate::core::segment::{delete_local_segment, segment_already_delete};
use crate::core::shard::{delete_local_shard, is_delete_by_shard};
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

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
        let req = request.into_inner();
        let conf = journal_server_conf();
        if req.cluster_name != conf.cluster_name {
            return Ok(Response::new(UpdateJournalCacheReply::default()));
        }

        parse_notification(
            &self.cache_manager,
            &self.segment_file_manager,
            req.action_type(),
            req.resource_type(),
            &req.data,
        )
        .await;

        return Ok(Response::new(UpdateJournalCacheReply::default()));
    }

    async fn delete_shard_file(
        &self,
        request: Request<DeleteShardFileRequest>,
    ) -> Result<Response<DeleteShardFileReply>, Status> {
        let req = request.into_inner();
        let conf = journal_server_conf();
        if req.cluster_name != conf.cluster_name {
            return Ok(Response::new(DeleteShardFileReply::default()));
        }

        delete_local_shard(
            self.cache_manager.clone(),
            self.rocksdb_engine_handler.clone(),
            self.segment_file_manager.clone(),
            req,
        );

        return Ok(Response::new(DeleteShardFileReply::default()));
    }

    async fn get_shard_delete_status(
        &self,
        request: Request<GetShardDeleteStatusRequest>,
    ) -> Result<Response<GetShardDeleteStatusReply>, Status> {
        let req = request.into_inner();
        let conf = journal_server_conf();
        if req.cluster_name != conf.cluster_name {
            return Ok(Response::new(GetShardDeleteStatusReply::default()));
        }

        match is_delete_by_shard(&req) {
            Ok(flag) => {
                return Ok(Response::new(GetShardDeleteStatusReply { status: flag }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_segment_file(
        &self,
        request: Request<DeleteSegmentFileRequest>,
    ) -> Result<Response<DeleteSegmentFileReply>, Status> {
        let req = request.into_inner();
        let conf = journal_server_conf();
        if req.cluster_name != conf.cluster_name {
            return Ok(Response::new(DeleteSegmentFileReply::default()));
        }

        let segment_iden = SegmentIdentity::new(&req.namespace, &req.shard_name, req.segment);
        match delete_local_segment(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            &self.segment_file_manager,
            &segment_iden,
        )
        .await
        {
            Ok(()) => {
                return Ok(Response::new(DeleteSegmentFileReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get_segment_delete_status(
        &self,
        request: Request<GetSegmentDeleteStatusRequest>,
    ) -> Result<Response<GetSegmentDeleteStatusReply>, Status> {
        let req = request.into_inner();
        let conf = journal_server_conf();
        if req.cluster_name != conf.cluster_name {
            return Ok(Response::new(GetSegmentDeleteStatusReply::default()));
        }
        match segment_already_delete(&self.cache_manager, &req).await {
            Ok(flag) => {
                return Ok(Response::new(GetSegmentDeleteStatusReply { status: flag }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
