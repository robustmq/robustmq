use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    engine::{
        engine_service_client::EngineServiceClient, CreateSegmentRequest, CreateShardRequest,
        DeleteSegmentRequest, DeleteShardRequest,
    },
};
use tonic::transport::Channel;

pub(crate) async fn inner_create_shard(
    mut client: EngineServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match CreateShardRequest::decode(request.as_ref()) {
        Ok(request) => match client.create_shard(request).await {
            Ok(result) => {
                return Ok(CommonReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_delete_shard(
    mut client: EngineServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match DeleteShardRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete_shard(request).await {
            Ok(result) => {
                return Ok(CommonReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_create_segment(
    mut client: EngineServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match CreateSegmentRequest::decode(request.as_ref()) {
        Ok(request) => match client.create_segment(request).await {
            Ok(result) => {
                return Ok(CommonReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_delete_segment(
    mut client: EngineServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match DeleteSegmentRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete_segment(request).await {
            Ok(result) => {
                return Ok(CommonReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}
