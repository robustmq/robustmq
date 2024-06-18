use crate::{
    placement::{retry_call, PlacementCenterInterface, PlacementCenterService},
    poll::ClientPool,
};
use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    placement::{
        DeleteResourceConfigRequest, GetResourceConfigReply, GetResourceConfigRequest,
        HeartbeatRequest, RegisterNodeRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest,
        SendRaftMessageReply, SendRaftMessageRequest, SetResourceConfigRequest,
        UnRegisterNodeRequest,
    },
};
use std::sync::Arc;

pub async fn register_node(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: RegisterNodeRequest,
) -> Result<CommonReply, RobustMQError> {
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
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn un_register_node(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UnRegisterNodeRequest,
) -> Result<CommonReply, RobustMQError> {
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
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn heartbeat(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: HeartbeatRequest,
) -> Result<CommonReply, RobustMQError> {
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
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn send_raft_message(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendRaftMessageRequest,
) -> Result<SendRaftMessageReply, RobustMQError> {
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
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn send_raft_conf_change(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendRaftConfChangeRequest,
) -> Result<SendRaftConfChangeReply, RobustMQError> {
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
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn set_resource_config(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetResourceConfigRequest,
) -> Result<CommonReply, RobustMQError> {
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
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn delete_resource_config(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteResourceConfigRequest,
) -> Result<CommonReply, RobustMQError> {
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
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn get_resource_config(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: GetResourceConfigRequest,
) -> Result<GetResourceConfigReply, RobustMQError> {
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
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}
