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
use protocol::placement_center::generate::common::CommonReply;
use protocol::placement_center::generate::placement::{
    ClusterStatusReply, ClusterStatusRequest, DeleteIdempotentDataRequest,
    DeleteResourceConfigRequest, ExistsIdempotentDataReply, ExistsIdempotentDataRequest,
    GetResourceConfigReply, GetResourceConfigRequest, HeartbeatRequest, NodeListReply,
    NodeListRequest, RegisterNodeRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest,
    SendRaftMessageReply, SendRaftMessageRequest, SetIdempotentDataRequest,
    SetResourceConfigRequest, UnRegisterNodeRequest,
};

use crate::placement::{retry_call, PlacementCenterInterface, PlacementCenterService};
use crate::poll::ClientPool;

pub async fn cluster_status(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ClusterStatusRequest,
) -> Result<ClusterStatusReply, CommonError> {
    let request_data = ClusterStatusRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::ClusterStatus,
        client_poll,
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
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: NodeListRequest,
) -> Result<NodeListReply, CommonError> {
    let request_data = NodeListRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::ListNode,
        client_poll,
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
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: RegisterNodeRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = RegisterNodeRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::RegisterNode,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn un_register_node(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UnRegisterNodeRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = UnRegisterNodeRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::UnRegisterNode,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn heartbeat(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: HeartbeatRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = HeartbeatRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::Heartbeat,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn send_raft_message(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendRaftMessageRequest,
) -> Result<SendRaftMessageReply, CommonError> {
    let request_data = SendRaftMessageRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::SendRaftMessage,
        client_poll,
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
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendRaftConfChangeRequest,
) -> Result<SendRaftConfChangeReply, CommonError> {
    let request_data = SendRaftConfChangeRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::SendRaftConfChange,
        client_poll,
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
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetResourceConfigRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = SetResourceConfigRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::SetReourceConfig,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn delete_resource_config(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteResourceConfigRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteResourceConfigRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::DeleteReourceConfig,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn get_resource_config(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: GetResourceConfigRequest,
) -> Result<GetResourceConfigReply, CommonError> {
    let request_data = GetResourceConfigRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::GetReourceConfig,
        client_poll,
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
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetIdempotentDataRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = SetIdempotentDataRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::SetIdempotentData,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn delete_idempotent_data(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteIdempotentDataRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteIdempotentDataRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::DeleteIdempotentData,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn exists_idempotent_data(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ExistsIdempotentDataRequest,
) -> Result<ExistsIdempotentDataReply, CommonError> {
    let request_data = ExistsIdempotentDataRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Placement,
        PlacementCenterInterface::ExistsIdempotentData,
        client_poll,
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
