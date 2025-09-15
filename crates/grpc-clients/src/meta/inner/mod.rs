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
use protocol::meta::placement_center_inner::placement_center_service_client::PlacementCenterServiceClient;
use protocol::meta::placement_center_inner::{
    BindSchemaReply, BindSchemaRequest, ClusterStatusReply, ClusterStatusRequest,
    CreateSchemaReply, CreateSchemaRequest, DeleteIdempotentDataReply, DeleteIdempotentDataRequest,
    DeleteResourceConfigReply, DeleteResourceConfigRequest, DeleteSchemaReply, DeleteSchemaRequest,
    ExistsIdempotentDataReply, ExistsIdempotentDataRequest, GetOffsetDataReply,
    GetOffsetDataRequest, GetResourceConfigReply, GetResourceConfigRequest, HeartbeatReply,
    HeartbeatRequest, ListBindSchemaReply, ListBindSchemaRequest, ListSchemaReply,
    ListSchemaRequest, NodeListReply, NodeListRequest, RegisterNodeReply, RegisterNodeRequest,
    SaveOffsetDataReply, SaveOffsetDataRequest, SetIdempotentDataReply, SetIdempotentDataRequest,
    SetResourceConfigReply, SetResourceConfigRequest, UnBindSchemaReply, UnBindSchemaRequest,
    UnRegisterNodeReply, UnRegisterNodeRequest, UpdateSchemaReply, UpdateSchemaRequest,
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
    type Connection = PlacementCenterServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match PlacementCenterServiceClient::connect(format!("http://{}", self.addr.clone())).await {
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
    PlacementCenterServiceClient<Channel>,
    ClusterStatusReply,
    placement_center_inner_services_client,
    cluster_status,
    true
);

impl_retriable_request!(
    NodeListRequest,
    PlacementCenterServiceClient<Channel>,
    NodeListReply,
    placement_center_inner_services_client,
    node_list,
    true
);

impl_retriable_request!(
    RegisterNodeRequest,
    PlacementCenterServiceClient<Channel>,
    RegisterNodeReply,
    placement_center_inner_services_client,
    register_node,
    true
);

impl_retriable_request!(
    UnRegisterNodeRequest,
    PlacementCenterServiceClient<Channel>,
    UnRegisterNodeReply,
    placement_center_inner_services_client,
    un_register_node,
    true
);

impl_retriable_request!(
    HeartbeatRequest,
    PlacementCenterServiceClient<Channel>,
    HeartbeatReply,
    placement_center_inner_services_client,
    heartbeat,
    true
);

impl_retriable_request!(
    SetResourceConfigRequest,
    PlacementCenterServiceClient<Channel>,
    SetResourceConfigReply,
    placement_center_inner_services_client,
    set_resource_config,
    true
);

impl_retriable_request!(
    GetResourceConfigRequest,
    PlacementCenterServiceClient<Channel>,
    GetResourceConfigReply,
    placement_center_inner_services_client,
    get_resource_config,
    true
);

impl_retriable_request!(
    DeleteResourceConfigRequest,
    PlacementCenterServiceClient<Channel>,
    DeleteResourceConfigReply,
    placement_center_inner_services_client,
    delete_resource_config,
    true
);

impl_retriable_request!(
    SetIdempotentDataRequest,
    PlacementCenterServiceClient<Channel>,
    SetIdempotentDataReply,
    placement_center_inner_services_client,
    set_idempotent_data,
    true
);

impl_retriable_request!(
    ExistsIdempotentDataRequest,
    PlacementCenterServiceClient<Channel>,
    ExistsIdempotentDataReply,
    placement_center_inner_services_client,
    exists_idempotent_data,
    true
);

impl_retriable_request!(
    DeleteIdempotentDataRequest,
    PlacementCenterServiceClient<Channel>,
    DeleteIdempotentDataReply,
    placement_center_inner_services_client,
    delete_idempotent_data,
    true
);

impl_retriable_request!(
    SaveOffsetDataRequest,
    PlacementCenterServiceClient<Channel>,
    SaveOffsetDataReply,
    placement_center_inner_services_client,
    save_offset_data,
    true
);

impl_retriable_request!(
    GetOffsetDataRequest,
    PlacementCenterServiceClient<Channel>,
    GetOffsetDataReply,
    placement_center_inner_services_client,
    get_offset_data,
    true
);

impl_retriable_request!(
    ListSchemaRequest,
    PlacementCenterServiceClient<Channel>,
    ListSchemaReply,
    placement_center_inner_services_client,
    list_schema,
    true
);

impl_retriable_request!(
    CreateSchemaRequest,
    PlacementCenterServiceClient<Channel>,
    CreateSchemaReply,
    placement_center_inner_services_client,
    create_schema,
    true
);

impl_retriable_request!(
    UpdateSchemaRequest,
    PlacementCenterServiceClient<Channel>,
    UpdateSchemaReply,
    placement_center_inner_services_client,
    update_schema,
    true
);

impl_retriable_request!(
    DeleteSchemaRequest,
    PlacementCenterServiceClient<Channel>,
    DeleteSchemaReply,
    placement_center_inner_services_client,
    delete_schema,
    true
);

impl_retriable_request!(
    ListBindSchemaRequest,
    PlacementCenterServiceClient<Channel>,
    ListBindSchemaReply,
    placement_center_inner_services_client,
    list_bind_schema,
    true
);

impl_retriable_request!(
    BindSchemaRequest,
    PlacementCenterServiceClient<Channel>,
    BindSchemaReply,
    placement_center_inner_services_client,
    bind_schema,
    true
);

impl_retriable_request!(
    UnBindSchemaRequest,
    PlacementCenterServiceClient<Channel>,
    UnBindSchemaReply,
    placement_center_inner_services_client,
    un_bind_schema,
    true
);
