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

use metadata_struct::journal::segment::JournalSegment;
use openraft::raft::ClientWriteResponse;
use prost::Message;
use protocol::placement_center::placement_center_journal::engine_service_server::EngineService;
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};
use rocksdb_engine::RocksDBEngine;
use tonic::{Request, Response, Status};

use crate::cache::journal::JournalCacheManager;
use crate::cache::placement::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::raft::raftv2::typeconfig::TypeConfig;
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::journal::segment::{is_seal_up_segment, SegmentStorage};
use crate::storage::journal::shard::ShardStorage;

pub struct GrpcEngineService {
    raft_machine_apply: Arc<RaftMachineApply>,
    engine_cache: Arc<JournalCacheManager>,
    cluster_cache: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcEngineService {
    pub fn new(
        raft_machine_apply: Arc<RaftMachineApply>,
        engine_cache: Arc<JournalCacheManager>,
        cluster_cache: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcEngineService {
            raft_machine_apply,
            engine_cache,
            cluster_cache,
            rocksdb_engine_handler,
        }
    }
}
impl GrpcEngineService {
    fn parse_replicas_vec(
        &self,
        resp: ClientWriteResponse<TypeConfig>,
    ) -> Result<JournalSegment, PlacementCenterError> {
        if let Some(value) = resp.data.value {
            let data = serde_json::from_slice::<JournalSegment>(&value)?;
            return Ok(data);
        }
        Err(PlacementCenterError::ExecutionResultIsEmpty)
    }
}

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

        // Parameter validation

        // Check if the cluster exists
        if !self
            .cluster_cache
            .cluster_list
            .contains_key(&req.cluster_name)
        {
            return Err(Status::cancelled(
                PlacementCenterError::ClusterDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        // Check that the number of available nodes in the cluster is sufficient
        let num = self.cluster_cache.get_broker_num(&req.cluster_name) as u32;

        if num < req.replica {
            return Err(Status::cancelled(
                PlacementCenterError::NotEnoughNodes(req.replica, num).to_string(),
            ));
        }

        let shard_res =
            self.engine_cache
                .get_shard(&req.cluster_name, &req.namespace, &req.shard_name);

        // Handle situations where the Shard already exists in order to support interface idempotency
        if shard_res.is_some() {
            let shard = shard_res.unwrap();
            if let Some(segment) = self.engine_cache.get_segment(
                &req.cluster_name,
                &req.namespace,
                &req.shard_name,
                shard.active_segment_seq,
            ) {
                if !is_seal_up_segment(segment.status) {
                    let replicas = segment.replicas.iter().map(|rep| rep.node_id).collect();
                    return Ok(Response::new(CreateShardReply {
                        segment_no: shard.active_segment_seq,
                        replica: replicas,
                    }));
                }
            }

            let create_next_segment_request = CreateNextSegmentRequest {
                cluster_name: req.cluster_name.clone(),
                namespace: req.namespace.clone(),
                shard_name: req.shard_name.clone(),
                active_segment_next_num: 1,
            };

            let data = StorageData::new(
                StorageDataType::JournalCreateNextSegment,
                CreateNextSegmentRequest::encode_to_vec(&create_next_segment_request),
            );
            match self.raft_machine_apply.client_write(data).await {
                Ok(Some(resp)) => match self.parse_replicas_vec(resp) {
                    Ok(segment) => {
                        let replica: Vec<u64> =
                            segment.replicas.iter().map(|rep| rep.node_id).collect();

                        return Ok(Response::new(CreateShardReply {
                            segment_no: segment.segment_seq,
                            replica,
                        }));
                    }
                    Err(e) => {
                        return Err(Status::cancelled(e.to_string()));
                    }
                },
                Ok(None) => {
                    return Err(Status::cancelled(
                        PlacementCenterError::ExecutionResultIsEmpty.to_string(),
                    ));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }

        // Create a new Shard
        let data = StorageData::new(
            StorageDataType::JournalCreateShard,
            CreateShardRequest::encode_to_vec(&req),
        );
        match self.raft_machine_apply.client_write(data).await {
            Ok(Some(resp)) => match self.parse_replicas_vec(resp) {
                Ok(segment) => {
                    let replica: Vec<u64> =
                        segment.replicas.iter().map(|rep| rep.node_id).collect();

                    return Ok(Response::new(CreateShardReply {
                        segment_no: segment.segment_seq,
                        replica,
                    }));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            },
            Ok(None) => {
                return Err(Status::cancelled(
                    PlacementCenterError::ExecutionResultIsEmpty.to_string(),
                ));
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

        if !self
            .cluster_cache
            .cluster_list
            .contains_key(&req.cluster_name)
        {
            return Err(Status::cancelled(
                PlacementCenterError::ClusterDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        let shard = self
            .engine_cache
            .get_shard(&req.cluster_name, &req.namespace, &req.shard_name);
        if shard.is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ShardDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalDeleteShard,
            DeleteShardRequest::encode_to_vec(&req),
        );
        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(DeleteShardReply::default())),
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
        let res = if req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == 0 {
            match segment_storage.list_by_cluster(&req.cluster_name) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else if !req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == 0 {
            match segment_storage.list_by_namespace(&req.cluster_name, &req.namespace) {
                Ok(list) => list,
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else if !req.namespace.is_empty() && !req.shard_name.is_empty() && req.segment_no == 0 {
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
                req.segment_no,
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

        if !self
            .cluster_cache
            .cluster_list
            .contains_key(&req.cluster_name)
        {
            return Err(Status::cancelled(
                PlacementCenterError::ClusterDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        let shard = self
            .engine_cache
            .get_shard(&req.cluster_name, &req.namespace, &req.shard_name);

        if shard.is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ShardDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalCreateNextSegment,
            CreateNextSegmentRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(Some(resp)) => match self.parse_replicas_vec(resp) {
                Ok(segment) => {
                    let replica: Vec<u64> =
                        segment.replicas.iter().map(|rep| rep.node_id).collect();

                    return Ok(Response::new(CreateNextSegmentReply {
                        segment_no: segment.segment_seq,
                        replica,
                    }));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            },
            Ok(None) => {
                return Err(Status::cancelled(
                    PlacementCenterError::ExecutionResultIsEmpty.to_string(),
                ));
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

        if !self
            .cluster_cache
            .cluster_list
            .contains_key(&req.cluster_name)
        {
            return Err(Status::cancelled(
                PlacementCenterError::ClusterDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        let shard = self
            .engine_cache
            .get_shard(&req.cluster_name, &req.namespace, &req.shard_name);

        if shard.is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ShardDoesNotExist(req.cluster_name).to_string(),
            ));
        }

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::JournalDeleteSegment,
            DeleteSegmentRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(DeleteSegmentReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
