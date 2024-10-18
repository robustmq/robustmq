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

use grpc_clients::poll::ClientPool;
use protocol::journal_server::journal_engine::{
    GetActiveSegmentReq, GetActiveSegmentRespShard, GetClusterMetadataNode, RespHeader, WriteReq,
    WriteRespMessage,
};

use super::cache::CacheManager;
use super::error::JournalServerError;
use super::shard::{create_active_segement, create_shard};

#[derive(Clone)]
pub struct Handler {
    cache_manager: Arc<CacheManager>,
    client_poll: Arc<ClientPool>,
}

impl Handler {
    pub fn new(cache_manager: Arc<CacheManager>, client_poll: Arc<ClientPool>) -> Handler {
        Handler {
            cache_manager,
            client_poll,
        }
    }

    pub fn get_cluster_metadata(&self) -> Vec<GetClusterMetadataNode> {
        let mut result = Vec::new();
        for (node_id, node) in self.cache_manager.node_list.clone() {
            result.push(GetClusterMetadataNode {
                replica_id: node_id as u32,
                replica_addr: node.node_inner_addr,
            });
        }
        result
    }

    pub async fn write(
        &self,
        request: WriteReq,
    ) -> Result<Vec<WriteRespMessage>, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty("write".to_string()));
        }

        let req_body = request.body.unwrap();

        // validator

        // write
        for message in req_body.messages {}

        Ok(Vec::new())
    }

    pub async fn read(&self) {}

    pub async fn active_segment(
        &self,
        request: GetActiveSegmentReq,
    ) -> Result<Vec<GetActiveSegmentRespShard>, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty(
                "active_segment".to_string(),
            ));
        }

        let req_body = request.body.unwrap();
        let mut results = Vec::new();

        let cluster = self.cache_manager.get_cluster();

        for raw in req_body.shards {
            let shard = if let Some(shard) = self
                .cache_manager
                .get_shard(&raw.namespace, &raw.shard_name)
            {
                shard
            } else {
                create_shard(self.client_poll.clone(), &raw.namespace, &raw.shard_name).await?
            };

            let active_segment = if let Some(segment) = self
                .cache_manager
                .get_active_segment(&raw.namespace, &raw.shard_name)
            {
                segment
            } else {
                create_active_segement(&raw.namespace, &raw.shard_name).await?
            };
            results.push(GetActiveSegmentRespShard {
                namespace: raw.namespace.clone(),
                shard: raw.namespace.clone(),
                replica_id: active_segment
                    .replica
                    .iter()
                    .map(|rep| rep.node_id)
                    .collect(),
            });
        }

        Ok(results)
    }

    pub async fn offset_commit(&self) {}

    fn resp_header(&self) -> Option<RespHeader> {
        None
    }
}
