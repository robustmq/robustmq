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

use crate::core::cache::MetaCacheManager;
use crate::core::cluster::{register_node_by_req, un_register_node_by_req};
use crate::raft::manager::MultiRaftManager;
use crate::raft::services::{
    add_learner_by_req, append_by_req, change_membership_by_req, snapshot_by_req, vote_by_req,
};
use crate::server::services::common::inner::{
    cluster_status_by_req, delete_resource_config_by_req, get_offset_data_by_req,
    get_resource_config_by_req, heartbeat_by_req, node_list_by_req, save_offset_data_by_req,
    set_resource_config_by_req,
};
use crate::server::services::common::kv::{
    delete_by_req, exists_by_req, get_by_req, get_prefix_by_req, set_by_req,
};
use crate::server::services::common::schema::{
    bind_schema_req, create_schema_req, delete_schema_req, list_bind_schema_req, list_schema_req,
    un_bind_schema_req, update_schema_req,
};
use crate::server::services::common::tenant::{
    create_tenant_by_req, delete_tenant_by_req, list_tenant_by_req, update_tenant_by_req,
};
use crate::server::services::mqtt::share_group::{
    add_share_group_member_by_req, create_share_group_by_req, delete_share_group_by_req,
    delete_share_group_member_by_req, list_share_group_by_req, list_share_group_member_by_req,
};
use grpc_clients::pool::ClientPool;
use node_call::NodeCallManager;
use prost_validate::Validator;
use protocol::meta::meta_service_common::meta_service_service_server::MetaServiceService;
use protocol::meta::meta_service_common::{
    AddLearnerReply, AddLearnerRequest, AddShareGroupMemberReply, AddShareGroupMemberRequest,
    AppendReply, AppendRequest, BindSchemaReply, BindSchemaRequest, ChangeMembershipReply,
    ChangeMembershipRequest, ClusterStatusReply, ClusterStatusRequest, CreateSchemaReply,
    CreateSchemaRequest, CreateShareGroupReply, CreateShareGroupRequest, CreateTenantReply,
    CreateTenantRequest, DeleteReply, DeleteRequest, DeleteResourceConfigReply,
    DeleteResourceConfigRequest, DeleteSchemaReply, DeleteSchemaRequest,
    DeleteShareGroupMemberReply, DeleteShareGroupMemberRequest, DeleteShareGroupReply,
    DeleteShareGroupRequest, DeleteTenantReply, DeleteTenantRequest, ExistsReply, ExistsRequest,
    GetOffsetDataReply, GetOffsetDataRequest, GetPrefixReply, GetPrefixRequest, GetReply,
    GetRequest, GetResourceConfigReply, GetResourceConfigRequest, HeartbeatReply, HeartbeatRequest,
    ListBindSchemaReply, ListBindSchemaRequest, ListSchemaReply, ListSchemaRequest,
    ListShareGroupMemberReply, ListShareGroupMemberRequest, ListShareGroupReply,
    ListShareGroupRequest, ListTenantReply, ListTenantRequest, NodeListReply, NodeListRequest,
    RegisterNodeReply, RegisterNodeRequest, ReportMonitorReply, ReportMonitorRequest,
    SaveOffsetDataReply, SaveOffsetDataRequest, SetReply, SetRequest, SetResourceConfigReply,
    SetResourceConfigRequest, SnapshotReply, SnapshotRequest, UnBindSchemaReply,
    UnBindSchemaRequest, UnRegisterNodeReply, UnRegisterNodeRequest, UpdateSchemaReply,
    UpdateSchemaRequest, UpdateTenantReply, UpdateTenantRequest, VoteReply, VoteRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::{Request, Response, Status};

pub struct GrpcPlacementService {
    raft_manager: Arc<MultiRaftManager>,
    cluster_cache: Arc<MetaCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    mqtt_call_manager: Arc<NodeCallManager>,
}

impl GrpcPlacementService {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cluster_cache: Arc<MetaCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_pool: Arc<ClientPool>,
        mqtt_call_manager: Arc<NodeCallManager>,
    ) -> Self {
        GrpcPlacementService {
            raft_manager,
            cluster_cache,
            rocksdb_engine_handler,
            client_pool,
            mqtt_call_manager,
        }
    }

    // Helper: Validate request and convert errors
    fn validate_request<T: Validator>(&self, req: &T) -> Result<(), Status> {
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))
    }

    // Helper: Convert MetaServiceError to Status
    fn to_status<E: ToString>(e: E) -> Status {
        Status::internal(e.to_string())
    }
}

#[tonic::async_trait]
impl MetaServiceService for GrpcPlacementService {
    type ListSchemaStream = Pin<Box<dyn Stream<Item = Result<ListSchemaReply, Status>> + Send>>;
    type ListBindSchemaStream =
        Pin<Box<dyn Stream<Item = Result<ListBindSchemaReply, Status>> + Send>>;
    type ListTenantStream = Pin<Box<dyn Stream<Item = Result<ListTenantReply, Status>> + Send>>;

    // Cluster
    async fn cluster_status(
        &self,
        request: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        cluster_status_by_req(&self.raft_manager)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Node
    async fn node_list(
        &self,
        request: Request<NodeListRequest>,
    ) -> Result<Response<NodeListReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        node_list_by_req(&self.cluster_cache, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> Result<Response<RegisterNodeReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        register_node_by_req(
            &self.cluster_cache,
            &self.raft_manager,
            &self.client_pool,
            &self.mqtt_call_manager,
            req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn un_register_node(
        &self,
        request: Request<UnRegisterNodeRequest>,
    ) -> Result<Response<UnRegisterNodeReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        un_register_node_by_req(
            &self.cluster_cache,
            &self.raft_manager,
            &self.client_pool,
            &self.mqtt_call_manager,
            req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    // Heartbeat
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        heartbeat_by_req(&self.cluster_cache, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Monitor
    async fn report_monitor(
        &self,
        request: Request<ReportMonitorRequest>,
    ) -> Result<Response<ReportMonitorReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        Ok(Response::new(ReportMonitorReply::default()))
    }

    // Resource Config
    async fn set_resource_config(
        &self,
        request: Request<SetResourceConfigRequest>,
    ) -> Result<Response<SetResourceConfigReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        set_resource_config_by_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn get_resource_config(
        &self,
        request: Request<GetResourceConfigRequest>,
    ) -> Result<Response<GetResourceConfigReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        get_resource_config_by_req(&self.rocksdb_engine_handler, req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn delete_resource_config(
        &self,
        request: Request<DeleteResourceConfigRequest>,
    ) -> Result<Response<DeleteResourceConfigReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_resource_config_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Offset
    async fn save_offset_data(
        &self,
        request: Request<SaveOffsetDataRequest>,
    ) -> Result<Response<SaveOffsetDataReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        save_offset_data_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn get_offset_data(
        &self,
        request: Request<GetOffsetDataRequest>,
    ) -> Result<Response<GetOffsetDataReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        get_offset_data_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Schema
    async fn list_schema(
        &self,
        request: Request<ListSchemaRequest>,
    ) -> Result<Response<Self::ListSchemaStream>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_schema_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_schema(
        &self,
        request: Request<CreateSchemaRequest>,
    ) -> Result<Response<CreateSchemaReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_schema_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &req,
            &self.rocksdb_engine_handler,
        )
        .await
        .map_err(Self::to_status)?;

        Ok(Response::new(CreateSchemaReply {}))
    }

    async fn update_schema(
        &self,
        request: Request<UpdateSchemaRequest>,
    ) -> Result<Response<UpdateSchemaReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        update_schema_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.mqtt_call_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)?;

        Ok(Response::new(UpdateSchemaReply {}))
    }

    async fn delete_schema(
        &self,
        request: Request<DeleteSchemaRequest>,
    ) -> Result<Response<DeleteSchemaReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_schema_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.mqtt_call_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)?;

        Ok(Response::new(DeleteSchemaReply {}))
    }

    async fn list_bind_schema(
        &self,
        request: Request<ListBindSchemaRequest>,
    ) -> Result<Response<Self::ListBindSchemaStream>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_bind_schema_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn bind_schema(
        &self,
        request: Request<BindSchemaRequest>,
    ) -> Result<Response<BindSchemaReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        bind_schema_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.mqtt_call_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)?;

        Ok(Response::new(BindSchemaReply {}))
    }

    async fn un_bind_schema(
        &self,
        request: Request<UnBindSchemaRequest>,
    ) -> Result<Response<UnBindSchemaReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        un_bind_schema_req(&self.raft_manager, &self.mqtt_call_manager, &req)
            .await
            .map_err(Self::to_status)?;

        Ok(Response::new(UnBindSchemaReply {}))
    }

    // Tenant Operations
    async fn create_tenant(
        &self,
        request: Request<CreateTenantRequest>,
    ) -> Result<Response<CreateTenantReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_tenant_by_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn update_tenant(
        &self,
        request: Request<UpdateTenantRequest>,
    ) -> Result<Response<UpdateTenantReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        update_tenant_by_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_tenant(
        &self,
        request: Request<DeleteTenantRequest>,
    ) -> Result<Response<DeleteTenantReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_tenant_by_req(
            &self.raft_manager,
            &self.mqtt_call_manager,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn list_tenant(
        &self,
        request: Request<ListTenantRequest>,
    ) -> Result<Response<Self::ListTenantStream>, Status> {
        let req = request.into_inner();

        list_tenant_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // KV Operations
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        set_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        get_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn exists(
        &self,
        request: Request<ExistsRequest>,
    ) -> Result<Response<ExistsReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        exists_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn get_prefix(
        &self,
        request: Request<GetPrefixRequest>,
    ) -> Result<Response<GetPrefixReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        get_prefix_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Raft Internal
    async fn append(
        &self,
        request: Request<AppendRequest>,
    ) -> Result<Response<AppendReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        append_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn snapshot(
        &self,
        request: Request<SnapshotRequest>,
    ) -> Result<Response<SnapshotReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        snapshot_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn add_learner(
        &self,
        request: Request<AddLearnerRequest>,
    ) -> Result<Response<AddLearnerReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        add_learner_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn change_membership(
        &self,
        request: Request<ChangeMembershipRequest>,
    ) -> Result<Response<ChangeMembershipReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        change_membership_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        vote_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn list_share_group(
        &self,
        request: Request<ListShareGroupRequest>,
    ) -> Result<Response<ListShareGroupReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;
        list_share_group_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn list_share_group_member(
        &self,
        request: Request<ListShareGroupMemberRequest>,
    ) -> Result<Response<ListShareGroupMemberReply>, Status> {
        let req = request.into_inner();
        list_share_group_member_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_share_group(
        &self,
        request: Request<CreateShareGroupRequest>,
    ) -> Result<Response<CreateShareGroupReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;
        create_share_group_by_req(
            &self.cluster_cache,
            &self.raft_manager,
            &self.rocksdb_engine_handler,
            &self.mqtt_call_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_share_group(
        &self,
        request: Request<DeleteShareGroupRequest>,
    ) -> Result<Response<DeleteShareGroupReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;
        delete_share_group_by_req(
            &self.raft_manager,
            &self.rocksdb_engine_handler,
            &self.mqtt_call_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn add_share_group_member(
        &self,
        request: Request<AddShareGroupMemberRequest>,
    ) -> Result<Response<AddShareGroupMemberReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;
        add_share_group_member_by_req(
            &self.cluster_cache,
            &self.raft_manager,
            &self.rocksdb_engine_handler,
            &self.mqtt_call_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_share_group_member(
        &self,
        request: Request<DeleteShareGroupMemberRequest>,
    ) -> Result<Response<DeleteShareGroupMemberReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;
        delete_share_group_member_by_req(
            &self.raft_manager,
            &self.rocksdb_engine_handler,
            &self.mqtt_call_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }
}
