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
    CreateShardReq, DeleteShardReq, GetActiveSegmentReq, GetActiveSegmentRespShard,
};
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentRequest, CreateShardRequest, DeleteShardRequest,
};

use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;

#[derive(Clone)]
pub struct ShardHandler {
    cache_manager: Arc<CacheManager>,
    client_poll: Arc<ClientPool>,
}

impl ShardHandler {
    pub fn new(cache_manager: Arc<CacheManager>, client_poll: Arc<ClientPool>) -> ShardHandler {
        ShardHandler {
            cache_manager,
            client_poll,
        }
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
            let conf = journal_server_conf();
            let request = CreateShardRequest {
                cluster_name: conf.cluster_name.to_string(),
                namespace: req_body.namespace.to_string(),
                shard_name: req_body.shard_name.to_string(),
                replica: req_body.replica_num,
            };
            let reply = grpc_clients::placement::journal::call::create_shard(
                self.client_poll.clone(),
                conf.placement_center.clone(),
                request,
            )
            .await?;
            return Ok(reply.replica);
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
            .replicas
            .iter()
            .map(|replica| replica.node_id)
            .collect();
        Ok(replica_ids)
    }

    pub async fn delete_shard(&self, request: DeleteShardReq) -> Result<(), JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty(
                "create_shard".to_string(),
            ));
        }
        let req_body = request.body.unwrap();

        let shard = self
            .cache_manager
            .get_shard(&req_body.namespace, &req_body.shard_name);

        if shard.is_none() {
            return Err(JournalServerError::ShardNotExist(req_body.shard_name));
        }

        let conf = journal_server_conf();
        let request = DeleteShardRequest {
            cluster_name: conf.cluster_name.clone(),
            namespace: req_body.namespace,
            shard_name: req_body.shard_name,
        };

        grpc_clients::placement::journal::call::delete_shard(
            self.client_poll.clone(),
            conf.placement_center.clone(),
            request,
        )
        .await?;
        Ok(())
    }

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
                return Err(JournalServerError::ShardNotExist(raw.shard_name));
            };

            let resp_segment = if let Some(segment) = self
                .cache_manager
                .get_active_segment(&raw.namespace, &raw.shard_name)
            {
                GetActiveSegmentRespShard {
                    namespace: raw.namespace.clone(),
                    shard: raw.namespace.clone(),
                    replica_id: segment.replicas.iter().map(|rep| rep.node_id).collect(),
                }
            } else {
                let conf = journal_server_conf();
                let request = CreateNextSegmentRequest {
                    cluster_name: conf.cluster_name.to_string(),
                    namespace: raw.namespace.to_string(),
                    shard_name: raw.shard_name.to_string(),
                    active_segment_next_num: 1,
                };
                let reply = grpc_clients::placement::journal::call::create_next_segment(
                    self.client_poll.clone(),
                    conf.placement_center.clone(),
                    request,
                )
                .await?;
                GetActiveSegmentRespShard {
                    namespace: raw.namespace.clone(),
                    shard: raw.namespace.clone(),
                    replica_id: reply.replica,
                }
            };
            results.push(resp_segment);
        }

        Ok(results)
    }
}
