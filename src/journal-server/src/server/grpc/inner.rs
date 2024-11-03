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

use common_base::config::journal_server::journal_server_conf;
use protocol::journal_server::journal_inner::journal_server_inner_service_server::JournalServerInnerService;
use protocol::journal_server::journal_inner::{UpdateJournalCacheReply, UpdateJournalCacheRequest};
use rocksdb_engine::RocksDBEngine;
use tonic::{Request, Response, Status};

use crate::core::cache::{update_cache, CacheManager};
use crate::segment::manager::SegmentFileManager;

pub struct GrpcJournalServerInnerService {
    cache_manager: Arc<CacheManager>,
    segement_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcJournalServerInnerService {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        segement_file_manager: Arc<SegmentFileManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcJournalServerInnerService {
            cache_manager,
            segement_file_manager,
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
        update_cache(
            &self.cache_manager,
            &self.segement_file_manager,
            &self.rocksdb_engine_handler,
            req.action_type(),
            req.resource_type(),
            &req.data,
        )
        .await;

        return Ok(Response::new(UpdateJournalCacheReply::default()));
    }
}
