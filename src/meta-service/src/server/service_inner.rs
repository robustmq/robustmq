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

use crate::controller::journal::call_node::JournalInnerCallManager;
use crate::controller::mqtt::call_broker::MQTTInnerCallManager;
use crate::core::cache::CacheManager;
use crate::core::cluster::{register_node_by_req, un_register_node_by_req};
use crate::raft::manager::MultiRaftManager;
use crate::server::services::inner::{
    cluster_status_by_req, delete_resource_config_by_req, get_offset_data_by_req,
    get_resource_config_by_req, heartbeat_by_req, node_list_by_req, save_offset_data_by_req,
    set_resource_config_by_req,
};
use crate::server::services::schema::{
    bind_schema_req, create_schema_req, delete_schema_req, list_bind_schema_req, list_schema_req,
    un_bind_schema_req, update_schema_req,
};
use grpc_clients::pool::ClientPool;
use prost_validate::Validator;
use protocol::meta::meta_service_inner::meta_service_service_server::MetaServiceService;
use protocol::meta::meta_service_inner::{
    BindSchemaReply, BindSchemaRequest, ClusterStatusReply, ClusterStatusRequest,
    CreateSchemaReply, CreateSchemaRequest, DeleteResourceConfigReply, DeleteResourceConfigRequest,
    DeleteSchemaReply, DeleteSchemaRequest, GetOffsetDataReply, GetOffsetDataRequest,
    GetResourceConfigReply, GetResourceConfigRequest, HeartbeatReply, HeartbeatRequest,
    ListBindSchemaReply, ListBindSchemaRequest, ListSchemaReply, ListSchemaRequest, NodeListReply,
    NodeListRequest, RegisterNodeReply, RegisterNodeRequest, ReportMonitorReply,
    ReportMonitorRequest, SaveOffsetDataReply, SaveOffsetDataRequest, SetResourceConfigReply,
    SetResourceConfigRequest, UnBindSchemaReply, UnBindSchemaRequest, UnRegisterNodeReply,
    UnRegisterNodeRequest, UpdateSchemaReply, UpdateSchemaRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcPlacementService {
    raft_manager: Arc<MultiRaftManager>,
    cluster_cache: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    journal_call_manager: Arc<JournalInnerCallManager>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
}

impl GrpcPlacementService {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cluster_cache: Arc<CacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_pool: Arc<ClientPool>,
        journal_call_manager: Arc<JournalInnerCallManager>,
        mqtt_call_manager: Arc<MQTTInnerCallManager>,
    ) -> Self {
        GrpcPlacementService {
            raft_manager,
            cluster_cache,
            rocksdb_engine_handler,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
        }
    }
}

#[tonic::async_trait]
impl MetaServiceService for GrpcPlacementService {
    async fn cluster_status(
        &self,
        _: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        cluster_status_by_req(&self.raft_manager)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn node_list(
        &self,
        request: Request<NodeListRequest>,
    ) -> Result<Response<NodeListReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        node_list_by_req(&self.cluster_cache, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> Result<Response<RegisterNodeReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        register_node_by_req(
            &self.cluster_cache,
            &self.raft_manager,
            &self.client_pool,
            &self.journal_call_manager,
            &self.mqtt_call_manager,
            req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn un_register_node(
        &self,
        request: Request<UnRegisterNodeRequest>,
    ) -> Result<Response<UnRegisterNodeReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        un_register_node_by_req(
            &self.cluster_cache,
            &self.raft_manager,
            &self.client_pool,
            &self.journal_call_manager,
            &self.mqtt_call_manager,
            req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        heartbeat_by_req(&self.cluster_cache, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn report_monitor(
        &self,
        _: Request<ReportMonitorRequest>,
    ) -> Result<Response<ReportMonitorReply>, Status> {
        Ok(Response::new(ReportMonitorReply::default()))
    }

    async fn set_resource_config(
        &self,
        request: Request<SetResourceConfigRequest>,
    ) -> Result<Response<SetResourceConfigReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        set_resource_config_by_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::cancelled(e.to_string()))
        .map(Response::new)
    }

    async fn get_resource_config(
        &self,
        request: Request<GetResourceConfigRequest>,
    ) -> Result<Response<GetResourceConfigReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        get_resource_config_by_req(&self.rocksdb_engine_handler, req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_resource_config(
        &self,
        request: Request<DeleteResourceConfigRequest>,
    ) -> Result<Response<DeleteResourceConfigReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        delete_resource_config_by_req(&self.raft_manager, &req)
            .await
            .map_err(|e| Status::cancelled(e.to_string()))
            .map(Response::new)
    }

    async fn save_offset_data(
        &self,
        request: Request<SaveOffsetDataRequest>,
    ) -> Result<Response<SaveOffsetDataReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        save_offset_data_by_req(&self.raft_manager, &req)
            .await
            .map_err(|e| Status::cancelled(e.to_string()))
            .map(Response::new)
    }

    async fn get_offset_data(
        &self,
        request: Request<GetOffsetDataRequest>,
    ) -> Result<Response<GetOffsetDataReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        get_offset_data_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(|e| Status::cancelled(e.to_string()))
            .map(Response::new)
    }

    async fn list_schema(
        &self,
        request: Request<ListSchemaRequest>,
    ) -> Result<Response<ListSchemaReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        list_schema_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::cancelled(e.to_string()))
            .map(|data| ListSchemaReply { schemas: data })
            .map(Response::new)
    }

    async fn create_schema(
        &self,
        request: Request<CreateSchemaRequest>,
    ) -> Result<Response<CreateSchemaReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        create_schema_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
            &self.rocksdb_engine_handler,
        )
        .await
        .map_err(|e| Status::cancelled(e.to_string()))?;
        Ok(Response::new(CreateSchemaReply {}))
    }

    async fn update_schema(
        &self,
        request: Request<UpdateSchemaRequest>,
    ) -> Result<Response<UpdateSchemaReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        update_schema_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::cancelled(e.to_string()))?;
        Ok(Response::new(UpdateSchemaReply {}))
    }

    async fn delete_schema(
        &self,
        request: Request<DeleteSchemaRequest>,
    ) -> Result<Response<DeleteSchemaReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        delete_schema_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::cancelled(e.to_string()))?;
        Ok(Response::new(DeleteSchemaReply {}))
    }

    async fn list_bind_schema(
        &self,
        request: Request<ListBindSchemaRequest>,
    ) -> Result<Response<ListBindSchemaReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        list_bind_schema_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(|e| Status::cancelled(e.to_string()))
            .map(|schema_binds| ListBindSchemaReply { schema_binds })
            .map(Response::new)
    }

    async fn bind_schema(
        &self,
        request: Request<BindSchemaRequest>,
    ) -> Result<Response<BindSchemaReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        bind_schema_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::cancelled(e.to_string()))?;
        Ok(Response::new(BindSchemaReply {}))
    }

    async fn un_bind_schema(
        &self,
        request: Request<UnBindSchemaRequest>,
    ) -> Result<Response<UnBindSchemaReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        un_bind_schema_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::cancelled(e.to_string()))?;
        Ok(Response::new(UnBindSchemaReply {}))
    }
}
