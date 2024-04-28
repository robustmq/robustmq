use crate::placement::PlacementCenterService;
use crate::{
    placement::{retry_call, PlacementCenterInterface},
    ClientPool,
};
use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::journal::{CreateSegmentRequest, DeleteSegmentRequest, DeleteShardRequest};
use protocol::placement_center::generate::{common::CommonReply, journal::CreateShardRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn create_shard(
    client_poll: Arc<Mutex<ClientPool>>,
    addrs: Vec<String>,
    request: CreateShardRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = CreateShardRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::CreateShard,
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

pub async fn delete_shard(
    client_poll: Arc<Mutex<ClientPool>>,
    addrs: Vec<String>,
    request: DeleteShardRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteShardRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::DeleteShard,
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

pub async fn create_segment(
    client_poll: Arc<Mutex<ClientPool>>,
    addrs: Vec<String>,
    request: CreateSegmentRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = CreateSegmentRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::CreateSegment,
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

pub async fn delete_segment(
    client_poll: Arc<Mutex<ClientPool>>,
    addrs: Vec<String>,
    request: DeleteSegmentRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteSegmentRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::DeleteSegment,
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
