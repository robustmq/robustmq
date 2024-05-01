use crate::{
    placement::{retry_call, PlacementCenterInterface, PlacementCenterService}, poll::ClientPool,
};
use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    placement::{
        HeartbeatRequest, RegisterNodeRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest,
        SendRaftMessageReply, SendRaftMessageRequest, UnRegisterNodeRequest,
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
