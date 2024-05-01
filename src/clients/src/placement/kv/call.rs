use super::PlacementCenterInterface;
use crate::{
    placement::{retry_call, PlacementCenterService}, poll::ClientPool,
};
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::{
    common::CommonReply,
    kv::{DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetRequest},
};
use std::sync::Arc;

pub async fn placement_set(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = SetRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Kv,
        PlacementCenterInterface::Set,
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

pub async fn placement_get(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: GetRequest,
) -> Result<GetReply, RobustMQError> {
    let request_data = GetRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Kv,
        PlacementCenterInterface::Get,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match GetReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_delete(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Kv,
        PlacementCenterInterface::Delete,
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

pub async fn placement_exists(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ExistsRequest,
) -> Result<ExistsReply, RobustMQError> {
    let request_data = ExistsRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Kv,
        PlacementCenterInterface::Exists,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ExistsReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}
