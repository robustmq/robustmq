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

use protocol::journal_server::journal_admin::journal_server_admin_service_server::JournalServerAdminService;
use protocol::journal_server::journal_admin::{
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};
use tonic::{Request, Response, Status};

use crate::core::cache::CacheManager;

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
        let req = request.into_inner();
        let mut shards = Vec::new();
        if req.shard_name.is_empty() {
            // Get all shard
            for shard in self.cache_manager.get_shards() {
                match serde_json::to_string(&shard) {
                    Ok(data) => {
                        shards.push(data);
                    }
                    Err(e) => {
                        return Err(Status::cancelled(e.to_string()));
                    }
                }
            }
        } else {
            // Get shard by name
            if let Some(shard) = self
                .cache_manager
                .get_shard(&req.namespace, &req.shard_name)
            {
                match serde_json::to_string(&shard) {
                    Ok(data) => {
                        shards.push(data);
                    }
                    Err(e) => {
                        return Err(Status::cancelled(e.to_string()));
                    }
                }
            }
        }
        return Ok(Response::new(ListShardReply { shards }));
    }

    async fn list_segment(
        &self,
        request: Request<ListSegmentRequest>,
    ) -> Result<Response<ListSegmentReply>, Status> {
        let req = request.into_inner();

        let mut segments = Vec::new();
        if req.segment_no == -1 {
            // get all segment by shard
            for segment in self
                .cache_manager
                .get_segment_list_by_shard(&req.namespace, &req.shard_name)
            {
                match serde_json::to_string(&segment) {
                    Ok(data) => {
                        segments.push(data);
                    }
                    Err(e) => {
                        return Err(Status::cancelled(e.to_string()));
                    }
                }
            }
        } else {
            // get segment
            if let Some(segment) = self.cache_manager.get_segment(
                &req.namespace,
                &req.shard_name,
                req.segment_no as u32,
            ) {
                match serde_json::to_string(&segment) {
                    Ok(data) => {
                        segments.push(data);
                    }
                    Err(e) => {
                        return Err(Status::cancelled(e.to_string()));
                    }
                }
            }
        }
        return Ok(Response::new(ListSegmentReply { segments }));
    }
}
