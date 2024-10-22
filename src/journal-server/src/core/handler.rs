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
use grpc_clients::poll::ClientPool;
use protocol::journal_server::journal_engine::{
    CreateShardReq, GetActiveSegmentReq, GetActiveSegmentRespShard, GetClusterMetadataNode,
    RespHeader, WriteReq, WriteRespMessage,
};
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentRequest, CreateShardRequest,
};

use super::cache::CacheManager;
use super::error::JournalServerError;

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
                replica_id: node_id,
                replica_addr: node.node_inner_addr,
            });
        }
        result
    }

    pub async fn create_shard(
        &self,
        request: CreateShardReq,
    ) -> Result<Vec<u64>, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty(
                "create_shard".to_string(),
            ));
        }
        let req_body = request.body.unwrap();

        let shard = if let Some(shard) = self
            .cache_manager
            .get_shard(&req_body.namespace, &req_body.shard_name)
        {
            shard
        } else {
            let cluster = self.cache_manager.get_cluster();
            if cluster.enable_auto_create_shard {
                let conf = journal_server_conf();
                let request = CreateShardRequest {
                    cluster_name: conf.cluster_name.to_string(),
                    namespace: req_body.namespace.to_string(),
                    shard_name: req_body.shard_name.to_string(),
                    replica: req_body.replica_num,
                    storage_model: req_body.storage_model().as_str_name().to_string(),
                };
                let reply = grpc_clients::placement::journal::call::create_shard(
                    self.client_poll.clone(),
                    conf.placement_center.clone(),
                    request,
                )
                .await?;
                return Ok(reply.replica);
            }
            return Err(JournalServerError::ShardNotExist(req_body.shard_name));
        };

        let segment = if let Some(segment) = self
            .cache_manager
            .get_active_segment(&req_body.namespace, &req_body.shard_name)
        {
            segment
        } else {
            let conf = journal_server_conf();
            let request = CreateNextSegmentRequest {
                cluster_name: conf.cluster_name.to_string(),
                namespace: req_body.namespace.to_string(),
                shard_name: req_body.shard_name.to_string(),
                active_segment_next_num: 1,
            };
            let reply = grpc_clients::placement::journal::call::create_next_segment(
                self.client_poll.clone(),
                conf.placement_center.clone(),
                request,
            )
            .await?;
            return Ok(reply.replica);
        };
        let replica_ids = segment
            .replica
            .iter()
            .map(|replica| replica.node_id)
            .collect();
        Ok(replica_ids)
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
        let results = Vec::new();

        let cluster = self.cache_manager.get_cluster();

        for raw in req_body.shards {
            // let shard = if let Some(shard) = self
            //     .cache_manager
            //     .get_shard(&raw.namespace, &raw.shard_name)
            // {
            //     shard
            // } else {
            //     try_get_or_create_shard(
            //         &self.cache_manager,
            //         self.client_poll,
            //         &raw.namespace,
            //         &raw.shard_name,
            //     )
            //     .await?
            // };

            // let active_segment = if let Some(segment) = self
            //     .cache_manager
            //     .get_active_segment(&raw.namespace, &raw.shard_name)
            // {
            //     segment
            // } else {
            //     create_active_segement(&raw.namespace, &raw.shard_name).await?
            // };
            // results.push(GetActiveSegmentRespShard {
            //     namespace: raw.namespace.clone(),
            //     shard: raw.namespace.clone(),
            //     replica_id: active_segment
            //         .replica
            //         .iter()
            //         .map(|rep| rep.node_id)
            //         .collect(),
            // });
        }

        Ok(results)
    }

    pub async fn offset_commit(&self) {}

    fn resp_header(&self) -> Option<RespHeader> {
        None
    }
}
