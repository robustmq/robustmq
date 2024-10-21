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

use protocol::journal_server::journal_inner::journal_server_inner_service_server::JournalServerInnerService;
use protocol::journal_server::journal_inner::{UpdateJournalCacheReply, UpdateJournalCacheRequest};
use tonic::{Request, Response, Status};

use crate::core::cache::CacheManager;

pub struct GrpcJournalServerInnerService {
    cache_manager: Arc<CacheManager>,
}

impl GrpcJournalServerInnerService {
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        GrpcJournalServerInnerService { cache_manager }
    }
}

#[tonic::async_trait]
impl JournalServerInnerService for GrpcJournalServerInnerService {
    async fn update_cache(
        &self,
        request: Request<UpdateJournalCacheRequest>,
    ) -> Result<Response<UpdateJournalCacheReply>, Status> {
        let req = request.into_inner();
        self.cache_manager
            .update_cache(req.action_type(), req.resource_type(), req.data);

        return Ok(Response::new(UpdateJournalCacheReply::default()));
    }
}
