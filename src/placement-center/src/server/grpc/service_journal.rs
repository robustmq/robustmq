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

use grpc_clients::pool::ClientPool;
use protocol::placement_center::placement_center_journal::engine_service_server::EngineService;
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    ListShardReply, ListShardRequest, UpdateSegmentMetaReply, UpdateSegmentMetaRequest,
    UpdateSegmentStatusReply, UpdateSegmentStatusRequest,
};
use rocksdb_engine::RocksDBEngine;
use tonic::{Request, Response, Status};

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::journal::cache::JournalCacheManager;
use crate::journal::controller::call_node::JournalInnerCallManager;
use crate::journal::services::segmet::{
    create_segment_by_req, delete_segment_by_req, update_segment_meta_req,
    update_segment_status_req,
};
use crate::journal::services::shard::{create_shard_by_req, delete_shard_by_req};
use crate::route::apply::RaftMachineApply;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use crate::storage::journal::shard::ShardStorage;

pub struct GrpcEngineService {
    raft_machine_apply: Arc<RaftMachineApply>,
    engine_cache: Arc<JournalCacheManager>,
    cluster_cache: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    call_manager: Arc<JournalInnerCallManager>,
    client_pool: Arc<ClientPool>,
}

impl GrpcEngineService {
    pub fn new(
        raft_machine_apply: Arc<RaftMachineApply>,
        engine_cache: Arc<JournalCacheManager>,
        cluster_cache: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        call_manager: Arc<JournalInnerCallManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        GrpcEngineService {
            raft_machine_apply,
            engine_cache,
            cluster_cache,
            rocksdb_engine_handler,
            call_manager,
            client_pool,
        }
    }
}
impl GrpcEngineService {}

#[tonic::async_trait]
impl EngineService for GrpcEngineService {
    async fn list_shard(
        &self,
        request: Request<ListShardRequest>,
    ) -> Result<Response<ListShardReply>, Status> {
        let req = request.into_inner();
        if req.cluster_name.is_empty() {
            return Err(Status::cancelled(
                PlacementCenterError::RequestParamsNotEmpty(req.cluster_name).to_string(),
            ));
        }

        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        let res = if req.namespace.is_empty() && req.shard_name.is_empty() {
            match shard_storage.list_by_cluster(&req.cluster_name) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else if !req.namespace.is_empty() && req.shard_name.is_empty() {
            match shard_storage.list_by_cluster_namespace(&req.cluster_name, &req.namespace) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match shard_storage.get(&req.cluster_name, &req.namespace, &req.shard_name) {
                Ok(Some(shard)) => vec![shard],
                Ok(None) => Vec::new(),
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        };

        let body = match serde_json::to_vec(&res) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        return Ok(Response::new(ListShardReply { shards: body }));
    }

    async fn create_shard(
        &self,
        request: Request<CreateShardRequest>,
    ) -> Result<Response<CreateShardReply>, Status> {
        let req = request.into_inner();

        if self.cluster_cache.get_cluster(&req.cluster_name).is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ClusterDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        match create_shard_by_req(
            &self.engine_cache,
            &self.cluster_cache,
            &self.raft_machine_apply,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        {
            Ok(data) => {
                return Ok(Response::new(data));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_shard(
        &self,
        request: Request<DeleteShardRequest>,
    ) -> Result<Response<DeleteShardReply>, Status> {
        let req = request.into_inner();

        if self.cluster_cache.get_cluster(&req.cluster_name).is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ClusterDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        match delete_shard_by_req(
            &self.raft_machine_apply,
            &self.engine_cache,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        {
            Ok(data) => {
                return Ok(Response::new(data));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn list_segment(
        &self,
        request: Request<ListSegmentRequest>,
    ) -> Result<Response<ListSegmentReply>, Status> {
        let req = request.into_inner();
        if req.cluster_name.is_empty() {
            return Err(Status::cancelled(
                PlacementCenterError::RequestParamsNotEmpty(req.cluster_name).to_string(),
            ));
        }

        let segment_storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        let res = if req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == -1 {
            match segment_storage.list_by_cluster(&req.cluster_name) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else if !req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == -1 {
            match segment_storage.list_by_namespace(&req.cluster_name, &req.namespace) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else if !req.namespace.is_empty() && !req.shard_name.is_empty() && req.segment_no == -1 {
            match segment_storage.list_by_shard(&req.cluster_name, &req.namespace, &req.shard_name)
            {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match segment_storage.get(
                &req.cluster_name,
                &req.namespace,
                &req.shard_name,
                req.segment_no as u32,
            ) {
                Ok(Some(shard)) => vec![shard],
                Ok(None) => Vec::new(),
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        };

        let body = match serde_json::to_vec(&res) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        return Ok(Response::new(ListSegmentReply { segments: body }));
    }

    async fn create_next_segment(
        &self,
        request: Request<CreateNextSegmentRequest>,
    ) -> Result<Response<CreateNextSegmentReply>, Status> {
        let req = request.into_inner();

        if self.cluster_cache.get_cluster(&req.cluster_name).is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ClusterDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        match create_segment_by_req(
            &self.engine_cache,
            &self.cluster_cache,
            &self.raft_machine_apply,
            &self.call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        {
            Ok(data) => {
                return Ok(Response::new(data));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_segment(
        &self,
        request: Request<DeleteSegmentRequest>,
    ) -> Result<Response<DeleteSegmentReply>, Status> {
        let req = request.into_inner();

        if self.cluster_cache.get_cluster(&req.cluster_name).is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ClusterDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        match delete_segment_by_req(
            &self.engine_cache,
            &self.raft_machine_apply,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        {
            Ok(data) => return Ok(Response::new(data)),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn update_segment_status(
        &self,
        request: Request<UpdateSegmentStatusRequest>,
    ) -> Result<Response<UpdateSegmentStatusReply>, Status> {
        let req = request.into_inner();
        if req.cluster_name.is_empty() {
            return Err(Status::cancelled(
                PlacementCenterError::RequestParamsNotEmpty(req.cluster_name).to_string(),
            ));
        }

        match update_segment_status_req(
            &self.engine_cache,
            &self.raft_machine_apply,
            &self.call_manager,
            &self.client_pool,
            req,
        )
        .await
        {
            Ok(()) => return Ok(Response::new(UpdateSegmentStatusReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn list_segment_meta(
        &self,
        request: Request<ListSegmentMetaRequest>,
    ) -> Result<Response<ListSegmentMetaReply>, Status> {
        let req = request.into_inner();
        if req.cluster_name.is_empty() {
            return Err(Status::cancelled(
                PlacementCenterError::RequestParamsNotEmpty(req.cluster_name).to_string(),
            ));
        }

        let storage = SegmentMetadataStorage::new(self.rocksdb_engine_handler.clone());
        let res = if req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == -1 {
            match storage.list_by_cluster(&req.cluster_name) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else if !req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == -1 {
            match storage.list_by_namespace(&req.cluster_name, &req.namespace) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else if !req.namespace.is_empty() && !req.shard_name.is_empty() && req.segment_no == -1 {
            match storage.list_by_shard(&req.cluster_name, &req.namespace, &req.shard_name) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match storage.get(
                &req.cluster_name,
                &req.namespace,
                &req.shard_name,
                req.segment_no as u32,
            ) {
                Ok(Some(shard)) => vec![shard],
                Ok(None) => Vec::new(),
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        };

        let body = match serde_json::to_vec(&res) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        return Ok(Response::new(ListSegmentMetaReply { segments: body }));
    }

    async fn update_segment_meta(
        &self,
        request: Request<UpdateSegmentMetaRequest>,
    ) -> Result<Response<UpdateSegmentMetaReply>, Status> {
        let req = request.into_inner();

        match update_segment_meta_req(
            &self.engine_cache,
            &self.raft_machine_apply,
            &self.call_manager,
            &self.client_pool,
            req,
        )
        .await
        {
            Ok(()) => return Ok(Response::new(UpdateSegmentMetaReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
