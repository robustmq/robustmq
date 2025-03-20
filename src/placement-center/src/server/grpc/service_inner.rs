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
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use log::{debug, info};
use prost::Message;
use prost_validate::Validator;
use protocol::placement_center::placement_center_inner::placement_center_service_server::PlacementCenterService;
use protocol::placement_center::placement_center_inner::{
    BindSchemaReply, BindSchemaRequest, ClusterStatusReply, ClusterStatusRequest,
    CreateSchemaReply, CreateSchemaRequest, DeleteIdempotentDataReply, DeleteIdempotentDataRequest,
    DeleteResourceConfigReply, DeleteResourceConfigRequest, DeleteSchemaReply, DeleteSchemaRequest,
    ExistsIdempotentDataReply, ExistsIdempotentDataRequest, GetOffsetDataReply,
    GetOffsetDataReplyOffset, GetOffsetDataRequest, GetResourceConfigReply,
    GetResourceConfigRequest, HeartbeatReply, HeartbeatRequest, ListBindSchemaReply,
    ListBindSchemaRequest, ListSchemaReply, ListSchemaRequest, NodeListReply, NodeListRequest,
    RegisterNodeReply, RegisterNodeRequest, ReportMonitorReply, ReportMonitorRequest,
    SaveOffsetDataReply, SaveOffsetDataRequest, SetIdempotentDataReply, SetIdempotentDataRequest,
    SetResourceConfigReply, SetResourceConfigRequest, UnBindSchemaReply, UnBindSchemaRequest,
    UnRegisterNodeReply, UnRegisterNodeRequest, UpdateSchemaReply, UpdateSchemaRequest,
};
use tonic::{Request, Response, Status};

use super::validate::ValidateExt;
use crate::core::cache::PlacementCacheManager;
use crate::core::cluster::{register_node_by_req, un_register_node_by_req};
use crate::core::error::PlacementCenterError;
use crate::core::schema::{
    bind_schema_req, create_schema_req, delete_schema_req, list_bind_schema_req, list_schema_req,
    un_bind_schema_req, update_schema_req,
};
use crate::journal::controller::call_node::JournalInnerCallManager;
use crate::mqtt::controller::call_broker::MQTTInnerCallManager;
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
    journal_call_manager: Arc<JournalInnerCallManager>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
}

impl GrpcPlacementService {
    pub fn new(
        raft_machine_apply: Arc<RaftMachineApply>,
        cluster_cache: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_pool: Arc<ClientPool>,
        journal_call_manager: Arc<JournalInnerCallManager>,
        mqtt_call_manager: Arc<MQTTInnerCallManager>,
    ) -> Self {
        GrpcPlacementService {
            raft_machine_apply,
            cluster_cache,
            rocksdb_engine_handler,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
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

        let _ = req.validate_ext()?;

        let mut nodes = Vec::new();

        for raw in self
            .cluster_cache
            .get_broker_node_by_cluster(&req.cluster_name)
        {
            nodes.push(raw.encode())
        }

        Ok(Response::new(NodeListReply { nodes }))
    }

    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> Result<Response<RegisterNodeReply>, Status> {
        let req = request.into_inner();

        let _ = req.validate_ext()?;

        info!("register node:{:?}", req);
        match register_node_by_req(
            &self.cluster_cache,
            &self.raft_machine_apply,
            &self.client_pool,
            &self.journal_call_manager,
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

        info!("un register node:{:?}", req);
        match un_register_node_by_req(
            &self.cluster_cache,
            &self.raft_machine_apply,
            &self.client_pool,
            &self.journal_call_manager,
            &self.mqtt_call_manager,
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

        debug!(
            "receive heartbeat from node:{:?},time:{}",
            req.node_id,
            now_second()
        );
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
            StorageDataType::ResourceConfigSet,
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

        let _ = req.validate_ext()?;

        let data = StorageData::new(
            StorageDataType::ResourceConfigDelete,
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
            StorageDataType::IdempotentDataSet,
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

        let _ = req.validate_ext()?;

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
            StorageDataType::IdempotentDataDelete,
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
            StorageDataType::OffsetSet,
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
        let _ = req
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
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

    async fn list_schema(
        &self,
        request: Request<ListSchemaRequest>,
    ) -> Result<Response<ListSchemaReply>, Status> {
        let req = request.into_inner();
        match list_schema_req(&self.rocksdb_engine_handler, &req) {
            Ok(data) => {
                return Ok(Response::new(ListSchemaReply { schemas: data }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn create_schema(
        &self,
        request: Request<CreateSchemaRequest>,
    ) -> Result<Response<CreateSchemaReply>, Status> {
        let req = request.into_inner();
        match create_schema_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        {
            Ok(_) => {
                return Ok(Response::new(CreateSchemaReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn update_schema(
        &self,
        request: Request<UpdateSchemaRequest>,
    ) -> Result<Response<UpdateSchemaReply>, Status> {
        let req = request.into_inner();
        match update_schema_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        {
            Ok(_) => {
                return Ok(Response::new(UpdateSchemaReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_schema(
        &self,
        request: Request<DeleteSchemaRequest>,
    ) -> Result<Response<DeleteSchemaReply>, Status> {
        let req = request.into_inner();
        match delete_schema_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        {
            Ok(_) => {
                return Ok(Response::new(DeleteSchemaReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn list_bind_schema(
        &self,
        request: Request<ListBindSchemaRequest>,
    ) -> Result<Response<ListBindSchemaReply>, Status> {
        let req = request.into_inner();
        match list_bind_schema_req(&self.rocksdb_engine_handler, &req).await {
            Ok(schema_binds) => {
                return Ok(Response::new(ListBindSchemaReply { schema_binds }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn bind_schema(
        &self,
        request: Request<BindSchemaRequest>,
    ) -> Result<Response<BindSchemaReply>, Status> {
        let req = request.into_inner();
        match bind_schema_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        {
            Ok(_) => {
                return Ok(Response::new(BindSchemaReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn un_bind_schema(
        &self,
        request: Request<UnBindSchemaRequest>,
    ) -> Result<Response<UnBindSchemaReply>, Status> {
        let req = request.into_inner();
        match un_bind_schema_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        {
            Ok(_) => {
                return Ok(Response::new(UnBindSchemaReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
