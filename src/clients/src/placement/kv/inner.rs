use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    kv::{
        kv_service_client::KvServiceClient, DeleteRequest, ExistsReply, ExistsRequest, GetReply,
        GetRequest, SetRequest,
    },
};
use tonic::transport::Channel;

pub(crate) async fn inner_get(
    mut client: KvServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match GetRequest::decode(request.as_ref()) {
        Ok(request) => match client.get(request).await {
            Ok(result) => {
                return Ok(GetReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_set(
    mut client: KvServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match SetRequest::decode(request.as_ref()) {
        Ok(request) => match client.set(request).await {
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

pub(crate) async fn inner_delete(
    mut client: KvServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match DeleteRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete(request).await {
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

pub(crate) async fn inner_exists(
    mut client: KvServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match ExistsRequest::decode(request.as_ref()) {
        Ok(request) => match client.exists(request).await {
            Ok(result) => {
                return Ok(ExistsReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}
