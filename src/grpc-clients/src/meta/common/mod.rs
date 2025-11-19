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

use common_base::error::common::CommonError;
use mobc::Manager;
use protocol::meta::meta_service_common::meta_service_service_client::MetaServiceServiceClient;
use protocol::meta::meta_service_common::{
    AddLearnerReply, AddLearnerRequest, AppendReply, AppendRequest, BindSchemaReply,
    BindSchemaRequest, ChangeMembershipReply, ChangeMembershipRequest, ClusterStatusReply,
    ClusterStatusRequest, CreateSchemaReply, CreateSchemaRequest, DeleteReply, DeleteRequest,
    DeleteResourceConfigReply, DeleteResourceConfigRequest, DeleteSchemaReply, DeleteSchemaRequest,
    ExistsReply, ExistsRequest, GetOffsetDataReply, GetOffsetDataRequest, GetPrefixReply,
    GetPrefixRequest, GetReply, GetRequest, GetResourceConfigReply, GetResourceConfigRequest,
    HeartbeatReply, HeartbeatRequest, ListBindSchemaReply, ListBindSchemaRequest, ListSchemaReply,
    ListSchemaRequest, NodeListReply, NodeListRequest, RegisterNodeReply, RegisterNodeRequest,
    SaveOffsetDataReply, SaveOffsetDataRequest, SetReply, SetRequest, SetResourceConfigReply,
    SetResourceConfigRequest, SnapshotReply, SnapshotRequest, UnBindSchemaReply,
    UnBindSchemaRequest, UnRegisterNodeReply, UnRegisterNodeRequest, UpdateSchemaReply,
    UpdateSchemaRequest, VoteReply, VoteRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

pub struct PlacementServiceManager {
    pub addr: String,
}

impl PlacementServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for PlacementServiceManager {
    type Connection = MetaServiceServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MetaServiceServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommonError(format!(
                    "manager connect error:{},{}",
                    err,
                    self.addr.clone()
                )))
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

impl_retriable_request!(
    ClusterStatusRequest,
    MetaServiceServiceClient<Channel>,
    ClusterStatusReply,
    meta_service_inner_services_client,
    cluster_status,
    "PlacementService",
    "ClusterStatus",
    true
);

impl_retriable_request!(
    NodeListRequest,
    MetaServiceServiceClient<Channel>,
    NodeListReply,
    meta_service_inner_services_client,
    node_list,
    "PlacementService",
    "NodeList",
    true
);

impl_retriable_request!(
    RegisterNodeRequest,
    MetaServiceServiceClient<Channel>,
    RegisterNodeReply,
    meta_service_inner_services_client,
    register_node,
    "PlacementService",
    "RegisterNode",
    true
);

impl_retriable_request!(
    UnRegisterNodeRequest,
    MetaServiceServiceClient<Channel>,
    UnRegisterNodeReply,
    meta_service_inner_services_client,
    un_register_node,
    "PlacementService",
    "UnRegisterNode",
    true
);

impl_retriable_request!(
    HeartbeatRequest,
    MetaServiceServiceClient<Channel>,
    HeartbeatReply,
    meta_service_inner_services_client,
    heartbeat,
    "PlacementService",
    "Heartbeat",
    true
);

impl_retriable_request!(
    SetResourceConfigRequest,
    MetaServiceServiceClient<Channel>,
    SetResourceConfigReply,
    meta_service_inner_services_client,
    set_resource_config,
    "PlacementService",
    "SetResourceConfig",
    true
);

impl_retriable_request!(
    GetResourceConfigRequest,
    MetaServiceServiceClient<Channel>,
    GetResourceConfigReply,
    meta_service_inner_services_client,
    get_resource_config,
    "PlacementService",
    "GetResourceConfig",
    true
);

impl_retriable_request!(
    DeleteResourceConfigRequest,
    MetaServiceServiceClient<Channel>,
    DeleteResourceConfigReply,
    meta_service_inner_services_client,
    delete_resource_config,
    "PlacementService",
    "DeleteResourceConfig",
    true
);

impl_retriable_request!(
    SaveOffsetDataRequest,
    MetaServiceServiceClient<Channel>,
    SaveOffsetDataReply,
    meta_service_inner_services_client,
    save_offset_data,
    "PlacementService",
    "SaveOffsetData",
    true
);

impl_retriable_request!(
    GetOffsetDataRequest,
    MetaServiceServiceClient<Channel>,
    GetOffsetDataReply,
    meta_service_inner_services_client,
    get_offset_data,
    "PlacementService",
    "GetOffsetData",
    true
);

impl_retriable_request!(
    ListSchemaRequest,
    MetaServiceServiceClient<Channel>,
    ListSchemaReply,
    meta_service_inner_services_client,
    list_schema,
    "PlacementService",
    "ListSchema",
    true
);

impl_retriable_request!(
    CreateSchemaRequest,
    MetaServiceServiceClient<Channel>,
    CreateSchemaReply,
    meta_service_inner_services_client,
    create_schema,
    "PlacementService",
    "CreateSchema",
    true
);

impl_retriable_request!(
    UpdateSchemaRequest,
    MetaServiceServiceClient<Channel>,
    UpdateSchemaReply,
    meta_service_inner_services_client,
    update_schema,
    "PlacementService",
    "UpdateSchema",
    true
);

impl_retriable_request!(
    DeleteSchemaRequest,
    MetaServiceServiceClient<Channel>,
    DeleteSchemaReply,
    meta_service_inner_services_client,
    delete_schema,
    "PlacementService",
    "DeleteSchema",
    true
);

impl_retriable_request!(
    ListBindSchemaRequest,
    MetaServiceServiceClient<Channel>,
    ListBindSchemaReply,
    meta_service_inner_services_client,
    list_bind_schema,
    "PlacementService",
    "ListBindSchema",
    true
);

impl_retriable_request!(
    BindSchemaRequest,
    MetaServiceServiceClient<Channel>,
    BindSchemaReply,
    meta_service_inner_services_client,
    bind_schema,
    "PlacementService",
    "BindSchema",
    true
);

impl_retriable_request!(
    UnBindSchemaRequest,
    MetaServiceServiceClient<Channel>,
    UnBindSchemaReply,
    meta_service_inner_services_client,
    un_bind_schema,
    "PlacementService",
    "UnBindSchema",
    true
);

impl_retriable_request!(
    SetRequest,
    MetaServiceServiceClient<Channel>,
    SetReply,
    meta_service_inner_services_client,
    set,
    "PlacementService",
    "Set",
    true
);

impl_retriable_request!(
    GetRequest,
    MetaServiceServiceClient<Channel>,
    GetReply,
    meta_service_inner_services_client,
    get,
    "PlacementService",
    "Get",
    true
);

impl_retriable_request!(
    DeleteRequest,
    MetaServiceServiceClient<Channel>,
    DeleteReply,
    meta_service_inner_services_client,
    delete,
    "PlacementService",
    "Delete",
    true
);

impl_retriable_request!(
    ExistsRequest,
    MetaServiceServiceClient<Channel>,
    ExistsReply,
    meta_service_inner_services_client,
    exists,
    "PlacementService",
    "Exists",
    true
);

impl_retriable_request!(
    GetPrefixRequest,
    MetaServiceServiceClient<Channel>,
    GetPrefixReply,
    meta_service_inner_services_client,
    get_prefix,
    "PlacementService",
    "GetPrefix",
    true
);

impl_retriable_request!(
    VoteRequest,
    MetaServiceServiceClient<Channel>,
    VoteReply,
    meta_service_inner_services_client,
    vote,
    "PlacementService",
    "Vote",
    true
);

impl_retriable_request!(
    AppendRequest,
    MetaServiceServiceClient<Channel>,
    AppendReply,
    meta_service_inner_services_client,
    append,
    "PlacementService",
    "Append",
    true
);

impl_retriable_request!(
    SnapshotRequest,
    MetaServiceServiceClient<Channel>,
    SnapshotReply,
    meta_service_inner_services_client,
    snapshot,
    "PlacementService",
    "Snapshot",
    true
);

impl_retriable_request!(
    AddLearnerRequest,
    MetaServiceServiceClient<Channel>,
    AddLearnerReply,
    meta_service_inner_services_client,
    add_learner,
    "PlacementService",
    "AddLearner",
    true
);

impl_retriable_request!(
    ChangeMembershipRequest,
    MetaServiceServiceClient<Channel>,
    ChangeMembershipReply,
    meta_service_inner_services_client,
    change_membership,
    "PlacementService",
    "ChangeMembership",
    true
);
