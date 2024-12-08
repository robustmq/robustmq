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

use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use prost::Message;
use protocol::placement_center::placement_center_inner::placement_center_service_server::PlacementCenterService;
use protocol::placement_center::placement_center_inner::{
    ClusterStatusReply, ClusterStatusRequest, DeleteIdempotentDataReply,
    DeleteIdempotentDataRequest, DeleteResourceConfigReply, DeleteResourceConfigRequest,
    ExistsIdempotentDataReply, ExistsIdempotentDataRequest, GetOffsetDataReply,
    GetOffsetDataReplyOffset, GetOffsetDataRequest, GetResourceConfigReply,
    GetResourceConfigRequest, HeartbeatReply, HeartbeatRequest, NodeListReply, NodeListRequest,
    RegisterNodeReply, RegisterNodeRequest, ReportMonitorReply, ReportMonitorRequest,
    SaveOffsetDataReply, SaveOffsetDataRequest, SetIdempotentDataReply, SetIdempotentDataRequest,
    SetResourceConfigReply, SetResourceConfigRequest, UnRegisterNodeReply, UnRegisterNodeRequest,
};
use tonic::{Request, Response, Status};

use super::validate::ValidateExt;
use crate::core::cache::PlacementCacheManager;
use crate::core::cluster::{register_node_by_req, un_register_node_by_req};
use crate::core::error::PlacementCenterError;
use crate::journal::controller::call_node::JournalInnerCallManager;
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::placement::config::ResourceConfigStorage;
use crate::storage::placement::idempotent::IdempotentStorage;
use crate::storage::placement::offset::OffsetStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub struct GrpcPlacementService {
    raft_machine_apply: Arc<RaftMachineApply>,
    cluster_cache: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    call_manager: Arc<JournalInnerCallManager>,
}

impl GrpcPlacementService {
    pub fn new(
        raft_machine_apply: Arc<RaftMachineApply>,
        cluster_cache: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_pool: Arc<ClientPool>,
        call_manager: Arc<JournalInnerCallManager>,
    ) -> Self {
        GrpcPlacementService {
            raft_machine_apply,
            cluster_cache,
            rocksdb_engine_handler,
            client_pool,
            call_manager,
        }
    }
}

#[tonic::async_trait]
impl PlacementCenterService for GrpcPlacementService {
    async fn cluster_status(
        &self,
        _: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        let mut reply = ClusterStatusReply::default();
        let status = self
            .raft_machine_apply
            .openraft_node
            .metrics()
            .borrow()
            .clone();

        reply.content = match serde_json::to_string(&status) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(
                    CommonError::CommonError(e.to_string()).to_string(),
                ));
            }
        };
        return Ok(Response::new(reply));
    }

    async fn node_list(
        &self,
        request: Request<NodeListRequest>,
    ) -> Result<Response<NodeListReply>, Status> {
        let req = request.into_inner();
        let mut nodes = Vec::new();
        if let Some(node_list) = self.cluster_cache.node_list.get(&req.cluster_name) {
            for raw in node_list.iter() {
                nodes.push(raw.value().encode())
            }
        }
        return Ok(Response::new(NodeListReply { nodes }));
    }

    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> Result<Response<RegisterNodeReply>, Status> {
        let req = request.into_inner();

        let _ = req.validate_ext()?;

        match register_node_by_req(
            &self.cluster_cache,
            &self.raft_machine_apply,
            &self.client_pool,
            &self.call_manager,
            req,
        )
        .await
        {
            Ok(()) => return Ok(Response::new(RegisterNodeReply::default())),
            Err(e) => {
                return Err(Status::internal(e.to_string()));
            }
        }
    }

    async fn un_register_node(
        &self,
        request: Request<UnRegisterNodeRequest>,
    ) -> Result<Response<UnRegisterNodeReply>, Status> {
        let req = request.into_inner();
        let _ = req.validate_ext()?;

        match un_register_node_by_req(
            &self.cluster_cache,
            &self.raft_machine_apply,
            &self.client_pool,
            &self.call_manager,
            req,
        )
        .await
        {
            Ok(()) => return Ok(Response::new(UnRegisterNodeReply::default())),
            Err(e) => {
                return Err(Status::internal(e.to_string()));
            }
        }
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatReply>, Status> {
        let req = request.into_inner();
        if self
            .cluster_cache
            .get_broker_node(&req.cluster_name, req.node_id)
            .is_none()
        {
            return Err(Status::internal(
                PlacementCenterError::NodeDoesNotExist(req.node_id).to_string(),
            ));
        }
        self.cluster_cache
            .report_broker_heart(&req.cluster_name, req.node_id);
        return Ok(Response::new(HeartbeatReply::default()));
    }

    async fn report_monitor(
        &self,
        _: Request<ReportMonitorRequest>,
    ) -> Result<Response<ReportMonitorReply>, Status> {
        return Ok(Response::new(ReportMonitorReply::default()));
    }

    async fn set_resource_config(
        &self,
        request: Request<SetResourceConfigRequest>,
    ) -> Result<Response<SetResourceConfigReply>, Status> {
        let req = request.into_inner();
        let _ = req.validate_ext()?;

        let data = StorageData::new(
            StorageDataType::ClusterSetResourceConfig,
            SetResourceConfigRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(SetResourceConfigReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get_resource_config(
        &self,
        request: Request<GetResourceConfigRequest>,
    ) -> Result<Response<GetResourceConfigReply>, Status> {
        let req = request.into_inner();
        let _ = req.validate_ext()?;

        let storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        match storage.get(req.cluster_name, req.resources) {
            Ok(data) => {
                if let Some(res) = data {
                    return Ok(Response::new(GetResourceConfigReply { config: res }));
                } else {
                    return Ok(Response::new(GetResourceConfigReply { config: Vec::new() }));
                }
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_resource_config(
        &self,
        request: Request<DeleteResourceConfigRequest>,
    ) -> Result<Response<DeleteResourceConfigReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::ClusterDeleteResourceConfig,
            DeleteResourceConfigRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(DeleteResourceConfigReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn set_idempotent_data(
        &self,
        request: Request<SetIdempotentDataRequest>,
    ) -> Result<Response<SetIdempotentDataReply>, Status> {
        let req = request.into_inner();
        let _ = req.validate_ext()?;
        let data = StorageData::new(
            StorageDataType::ClusterSetIdempotentData,
            SetIdempotentDataRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(SetIdempotentDataReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn exists_idempotent_data(
        &self,
        request: Request<ExistsIdempotentDataRequest>,
    ) -> Result<Response<ExistsIdempotentDataReply>, Status> {
        let req = request.into_inner();
        let storage = IdempotentStorage::new(self.rocksdb_engine_handler.clone());
        match storage.exists(&req.cluster_name, &req.producer_id, req.seq_num) {
            Ok(flag) => {
                return Ok(Response::new(ExistsIdempotentDataReply { exists: flag }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_idempotent_data(
        &self,
        request: Request<DeleteIdempotentDataRequest>,
    ) -> Result<Response<DeleteIdempotentDataReply>, Status> {
        let req = request.into_inner();
        let _ = req.validate_ext()?;
        let data = StorageData::new(
            StorageDataType::ClusterDeleteIdempotentData,
            DeleteIdempotentDataRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(DeleteIdempotentDataReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn save_offset_data(
        &self,
        request: Request<SaveOffsetDataRequest>,
    ) -> Result<Response<SaveOffsetDataReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::ClusterSaveOffset,
            SaveOffsetDataRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(SaveOffsetDataReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get_offset_data(
        &self,
        request: Request<GetOffsetDataRequest>,
    ) -> Result<Response<GetOffsetDataReply>, Status> {
        let req = request.into_inner();
        let offset_storage = OffsetStorage::new(self.rocksdb_engine_handler.clone());
        let offset_data = match offset_storage.group_offset(&req.cluster_name, &req.group) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        let mut results = Vec::new();
        for raw in offset_data {
            results.push(GetOffsetDataReplyOffset {
                namespace: raw.namespace,
                shard_name: raw.shard_name,
                offset: raw.offset,
            });
        }
        return Ok(Response::new(GetOffsetDataReply { offsets: results }));
    }
}
