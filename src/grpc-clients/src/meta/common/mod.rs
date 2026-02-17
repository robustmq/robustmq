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

impl_retriable_request!(
    ClusterStatusRequest,
    MetaServiceServiceClient<Channel>,
    ClusterStatusReply,
    cluster_status,
    "PlacementService",
    "ClusterStatus",
    true
);

impl_retriable_request!(
    NodeListRequest,
    MetaServiceServiceClient<Channel>,
    NodeListReply,
    node_list,
    "PlacementService",
    "NodeList",
    true
);

impl_retriable_request!(
    RegisterNodeRequest,
    MetaServiceServiceClient<Channel>,
    RegisterNodeReply,
    register_node,
    "PlacementService",
    "RegisterNode",
    true
);

impl_retriable_request!(
    UnRegisterNodeRequest,
    MetaServiceServiceClient<Channel>,
    UnRegisterNodeReply,
    un_register_node,
    "PlacementService",
    "UnRegisterNode",
    true
);

impl_retriable_request!(
    HeartbeatRequest,
    MetaServiceServiceClient<Channel>,
    HeartbeatReply,
    heartbeat,
    "PlacementService",
    "Heartbeat",
    true
);

impl_retriable_request!(
    SetResourceConfigRequest,
    MetaServiceServiceClient<Channel>,
    SetResourceConfigReply,
    set_resource_config,
    "PlacementService",
    "SetResourceConfig",
    true
);

impl_retriable_request!(
    GetResourceConfigRequest,
    MetaServiceServiceClient<Channel>,
    GetResourceConfigReply,
    get_resource_config,
    "PlacementService",
    "GetResourceConfig",
    true
);

impl_retriable_request!(
    DeleteResourceConfigRequest,
    MetaServiceServiceClient<Channel>,
    DeleteResourceConfigReply,
    delete_resource_config,
    "PlacementService",
    "DeleteResourceConfig",
    true
);

impl_retriable_request!(
    SaveOffsetDataRequest,
    MetaServiceServiceClient<Channel>,
    SaveOffsetDataReply,
    save_offset_data,
    "PlacementService",
    "SaveOffsetData",
    true
);

impl_retriable_request!(
    GetOffsetDataRequest,
    MetaServiceServiceClient<Channel>,
    GetOffsetDataReply,
    get_offset_data,
    "PlacementService",
    "GetOffsetData",
    true
);

impl_retriable_request!(
    ListSchemaRequest,
    MetaServiceServiceClient<Channel>,
    ListSchemaReply,
    list_schema,
    "PlacementService",
    "ListSchema",
    true
);

impl_retriable_request!(
    CreateSchemaRequest,
    MetaServiceServiceClient<Channel>,
    CreateSchemaReply,
    create_schema,
    "PlacementService",
    "CreateSchema",
    true
);

impl_retriable_request!(
    UpdateSchemaRequest,
    MetaServiceServiceClient<Channel>,
    UpdateSchemaReply,
    update_schema,
    "PlacementService",
    "UpdateSchema",
    true
);

impl_retriable_request!(
    DeleteSchemaRequest,
    MetaServiceServiceClient<Channel>,
    DeleteSchemaReply,
    delete_schema,
    "PlacementService",
    "DeleteSchema",
    true
);

impl_retriable_request!(
    ListBindSchemaRequest,
    MetaServiceServiceClient<Channel>,
    ListBindSchemaReply,
    list_bind_schema,
    "PlacementService",
    "ListBindSchema",
    true
);

impl_retriable_request!(
    BindSchemaRequest,
    MetaServiceServiceClient<Channel>,
    BindSchemaReply,
    bind_schema,
    "PlacementService",
    "BindSchema",
    true
);

impl_retriable_request!(
    UnBindSchemaRequest,
    MetaServiceServiceClient<Channel>,
    UnBindSchemaReply,
    un_bind_schema,
    "PlacementService",
    "UnBindSchema",
    true
);

impl_retriable_request!(
    SetRequest,
    MetaServiceServiceClient<Channel>,
    SetReply,
    set,
    "PlacementService",
    "Set",
    true
);

impl_retriable_request!(
    GetRequest,
    MetaServiceServiceClient<Channel>,
    GetReply,
    get,
    "PlacementService",
    "Get",
    true
);

impl_retriable_request!(
    DeleteRequest,
    MetaServiceServiceClient<Channel>,
    DeleteReply,
    delete,
    "PlacementService",
    "Delete",
    true
);

impl_retriable_request!(
    ExistsRequest,
    MetaServiceServiceClient<Channel>,
    ExistsReply,
    exists,
    "PlacementService",
    "Exists",
    true
);

impl_retriable_request!(
    GetPrefixRequest,
    MetaServiceServiceClient<Channel>,
    GetPrefixReply,
    get_prefix,
    "PlacementService",
    "GetPrefix",
    true
);

impl_retriable_request!(
    VoteRequest,
    MetaServiceServiceClient<Channel>,
    VoteReply,
    vote,
    "PlacementService",
    "Vote",
    true
);

impl_retriable_request!(
    AppendRequest,
    MetaServiceServiceClient<Channel>,
    AppendReply,
    append,
    "PlacementService",
    "Append",
    true
);

impl_retriable_request!(
    SnapshotRequest,
    MetaServiceServiceClient<Channel>,
    SnapshotReply,
    snapshot,
    "PlacementService",
    "Snapshot",
    true
);

impl_retriable_request!(
    AddLearnerRequest,
    MetaServiceServiceClient<Channel>,
    AddLearnerReply,
    add_learner,
    "PlacementService",
    "AddLearner",
    true
);

impl_retriable_request!(
    ChangeMembershipRequest,
    MetaServiceServiceClient<Channel>,
    ChangeMembershipReply,
    change_membership,
    "PlacementService",
    "ChangeMembership",
    true
);
