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
use grpc_clients::pool::ClientPool;
use protocol::journal_server::journal_engine::{
    ClientSegmentMetadata, CreateShardReq, DeleteShardReq, GetShardMetadataReq,
    GetShardMetadataRespShard,
};
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentRequest, CreateShardRequest, DeleteShardRequest,
};

use crate::core::cache::{load_metadata_cache, CacheManager};
use crate::core::error::JournalServerError;

#[derive(Clone)]
pub struct ShardHandler {
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
}

impl ShardHandler {
    pub fn new(cache_manager: Arc<CacheManager>, client_pool: Arc<ClientPool>) -> ShardHandler {
        ShardHandler {
            cache_manager,
            client_pool,
        }
    }

    pub async fn create_shard(&self, request: CreateShardReq) -> Result<(), JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty(
                "create_shard".to_string(),
            ));
        }
        let req_body = request.body.unwrap();

        if self
            .cache_manager
            .get_shard(&req_body.namespace, &req_body.shard_name)
            .is_none()
        {
            let conf = journal_server_conf();
            let request = CreateShardRequest {
                cluster_name: conf.cluster_name.to_string(),
                namespace: req_body.namespace.to_string(),
                shard_name: req_body.shard_name.to_string(),
                replica: req_body.replica_num,
            };
            let reply = grpc_clients::placement::journal::call::create_shard(
                self.client_pool.clone(),
                conf.placement_center.clone(),
                request,
            )
            .await?;
            return Ok(());
        };
        Ok(())
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
            self.client_pool.clone(),
            conf.placement_center.clone(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn get_shard_metadata(
        &self,
        request: GetShardMetadataReq,
    ) -> Result<Vec<GetShardMetadataRespShard>, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty(
                "active_segment".to_string(),
            ));
        }

        let req_body = request.body.unwrap();
        let mut results = Vec::new();

        for raw in req_body.shards {
            let shard = if let Some(shard) = self
                .cache_manager
                .get_shard(&raw.namespace, &raw.shard_name)
            {
                shard
            } else {
                // todo Is enable auto create shard
                return Err(JournalServerError::ShardNotExist(raw.shard_name));
            };

            // If the Shard has an active Segment,
            // it returns an error, triggers the client to retry, and triggers the server to create the next Segment.
            if self
                .cache_manager
                .get_active_segment(&raw.namespace, &raw.shard_name)
                .is_none()
            {
                self.trigger_create_next_segment(&raw.namespace, &raw.shard_name)
                    .await?;

                load_metadata_cache(&self.cache_manager, &self.client_pool).await;
                return Err(JournalServerError::NotActiveSegmet(shard.name()));
            };

            let key = self
                .cache_manager
                .shard_key(&raw.namespace, &raw.shard_name);

            let segments = if let Some(segments) = self.cache_manager.segments.get(&key) {
                segments
            } else {
                load_metadata_cache(&self.cache_manager, &self.client_pool).await;
                return Err(JournalServerError::NotAvailableSegmets(raw.shard_name));
            };

            let mut resp_shard_segments = Vec::new();
            for segment_raw in segments.iter() {
                let segment = segment_raw.value();

                let meta = if let Some(meta) = self.cache_manager.get_segment_meta(
                    &raw.namespace,
                    &raw.shard_name,
                    segment.segment_seq,
                ) {
                    meta
                } else {
                    load_metadata_cache(&self.cache_manager, &self.client_pool).await;
                    return Err(JournalServerError::SegmentMetaNotExists(raw.shard_name));
                };

                let meta_val = serde_json::to_vec(&meta)?;
                let client_segment_meta = ClientSegmentMetadata {
                    segment_no: segment.segment_seq,
                    replicas: segment.replicas.iter().map(|rep| rep.node_id).collect(),
                    meta: meta_val,
                };
                resp_shard_segments.push(client_segment_meta);
            }

            let resp_shard = GetShardMetadataRespShard {
                namespace: raw.namespace,
                shard: raw.shard_name,
                segments: resp_shard_segments,
            };

            results.push(resp_shard);
        }

        Ok(results)
    }

    async fn trigger_create_next_segment(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Result<(), JournalServerError> {
        let conf = journal_server_conf();
        let request = CreateNextSegmentRequest {
            cluster_name: conf.cluster_name.to_string(),
            namespace: namespace.to_string(),
            shard_name: shard_name.to_string(),
            active_segment_next_num: 1,
        };
        let reply = grpc_clients::placement::journal::call::create_next_segment(
            self.client_pool.clone(),
            conf.placement_center.clone(),
            request,
        )
        .await?;
        Ok(())
    }
}
