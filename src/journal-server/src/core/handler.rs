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

use common_base::error::journal_server::JournalServerError;
use metadata_struct::journal::segment::{JournalSegment, JournalSegmentNode};
use protocol::journal_server::journal_engine::{
    GetActiveSegmentReq, GetClusterMetadataNode, RespHeader,
};

use super::cache::CacheManager;
use super::shard::create_active_segement;

#[derive(Debug, Clone)]
pub struct Handler {
    cache_manager: Arc<CacheManager>,
}

impl Handler {
    pub fn new(cache_manager: Arc<CacheManager>) -> Handler {
        Handler { cache_manager }
    }

    pub fn get_cluster_metadata(&self) -> Vec<GetClusterMetadataNode> {
        Vec::new()
    }

    pub async fn write(&self) {}

    pub async fn read(&self) {}

    pub async fn active_segment(
        &self,
        request: GetActiveSegmentReq,
    ) -> Result<JournalSegment, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty(
                "active_segment".to_string(),
            ));
        }

        let req_body = request.body.unwrap();

        let active_segment = if let Some(segment) = self
            .cache_manager
            .get_active_segment(&req_body.namespace, &req_body.shard)
        {
            segment
        } else {
            create_active_segement(&req_body.namespace, &req_body.shard).await?
        };

        return Ok(active_segment);
    }

    pub async fn offset_commit(&self) {}

    fn resp_header(&self) -> Option<RespHeader> {
        return None;
    }
}
