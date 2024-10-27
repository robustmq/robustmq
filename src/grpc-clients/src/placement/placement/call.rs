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
use prost::Message;
use protocol::placement_center::placement_center_inner::{
    ClusterStatusReply, ClusterStatusRequest, DeleteIdempotentDataReply,
    DeleteIdempotentDataRequest, DeleteResourceConfigReply, DeleteResourceConfigRequest,
    ExistsIdempotentDataReply, ExistsIdempotentDataRequest, GetResourceConfigReply,
    GetResourceConfigRequest, HeartbeatReply, HeartbeatRequest, NodeListReply, NodeListRequest,
    RegisterNodeReply, RegisterNodeRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest,
    SendRaftMessageReply, SendRaftMessageRequest, SetIdempotentDataReply, SetIdempotentDataRequest,
    SetResourceConfigReply, SetResourceConfigRequest, UnRegisterNodeReply, UnRegisterNodeRequest,
};

use crate::placement::{retry_call, PlacementCenterInterface, PlacementCenterService};
use crate::pool::ClientPool;

pub async fn cluster_status(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ClusterStatusRequest,
) -> Result<ClusterStatusReply, CommonError> {
    let request_data = ClusterStatusRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::ClusterStatus,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ClusterStatusReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn node_list(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: NodeListRequest,
) -> Result<NodeListReply, CommonError> {
    let request_data = NodeListRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::ListNode,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match NodeListReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn register_node(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: RegisterNodeRequest,
) -> Result<RegisterNodeReply, CommonError> {
    let request_data = RegisterNodeRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::RegisterNode,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match RegisterNodeReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn unregister_node(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UnRegisterNodeRequest,
) -> Result<UnRegisterNodeReply, CommonError> {
    let request_data = UnRegisterNodeRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::UnRegisterNode,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match UnRegisterNodeReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn heartbeat(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: HeartbeatRequest,
) -> Result<HeartbeatReply, CommonError> {
    let request_data = HeartbeatRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::Heartbeat,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match HeartbeatReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn send_raft_message(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendRaftMessageRequest,
) -> Result<SendRaftMessageReply, CommonError> {
    let request_data = SendRaftMessageRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::SendRaftMessage,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match SendRaftMessageReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn send_raft_conf_change(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendRaftConfChangeRequest,
) -> Result<SendRaftConfChangeReply, CommonError> {
    let request_data = SendRaftConfChangeRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::SendRaftConfChange,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match SendRaftConfChangeReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn set_resource_config(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetResourceConfigRequest,
) -> Result<SetResourceConfigReply, CommonError> {
    let request_data = SetResourceConfigRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::SetReourceConfig,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match SetResourceConfigReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn delete_resource_config(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteResourceConfigRequest,
) -> Result<DeleteResourceConfigReply, CommonError> {
    let request_data = DeleteResourceConfigRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::DeleteReourceConfig,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match DeleteResourceConfigReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn get_resource_config(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: GetResourceConfigRequest,
) -> Result<GetResourceConfigReply, CommonError> {
    let request_data = GetResourceConfigRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::GetReourceConfig,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match GetResourceConfigReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn set_idempotent_data(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetIdempotentDataRequest,
) -> Result<SetIdempotentDataReply, CommonError> {
    let request_data = SetIdempotentDataRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::SetIdempotentData,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match SetIdempotentDataReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn delete_idempotent_data(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteIdempotentDataRequest,
) -> Result<DeleteIdempotentDataReply, CommonError> {
    let request_data = DeleteIdempotentDataRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::DeleteIdempotentData,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match DeleteIdempotentDataReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn exists_idempotent_data(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ExistsIdempotentDataRequest,
) -> Result<ExistsIdempotentDataReply, CommonError> {
    let request_data = ExistsIdempotentDataRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::ExistsIdempotentData,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ExistsIdempotentDataReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
