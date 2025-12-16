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

use crate::admin::services::{list_segment_by_req, list_shard_by_req};
use crate::core::cache::CacheManager;
use protocol::journal::journal_admin::journal_server_admin_service_server::JournalServerAdminService;
use protocol::journal::journal_admin::{
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcJournalServerAdminService {
    cache_manager: Arc<CacheManager>,
}

impl GrpcJournalServerAdminService {
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        GrpcJournalServerAdminService { cache_manager }
    }
}

#[tonic::async_trait]
impl JournalServerAdminService for GrpcJournalServerAdminService {
    async fn list_shard(
        &self,
        request: Request<ListShardRequest>,
    ) -> Result<Response<ListShardReply>, Status> {
        let request = request.into_inner();
        list_shard_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_segment(
        &self,
        request: Request<ListSegmentRequest>,
    ) -> Result<Response<ListSegmentReply>, Status> {
        let request = request.into_inner();
        list_segment_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }
}
