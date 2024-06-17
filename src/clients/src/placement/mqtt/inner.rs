use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        mqtt_service_client::MqttServiceClient, CreateTopicRequest, CreateUserRequest, DeleteTopicRequest, DeleteUserRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest, ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest
    },
};
use tonic::transport::Channel;

pub(crate) async fn inner_get_share_sub_leader(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match GetShareSubLeaderRequest::decode(request.as_ref()) {
        Ok(request) => match client.get_share_sub_leader(request).await {
            Ok(result) => {
                return Ok(GetShareSubLeaderReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_create_user(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match CreateUserRequest::decode(request.as_ref()) {
        Ok(request) => match client.create_user(request).await {
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

pub(crate) async fn inner_list_user(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match ListUserRequest::decode(request.as_ref()) {
        Ok(request) => match client.list_user(request).await {
            Ok(result) => {
                return Ok(ListUserReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_delete_user(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match DeleteUserRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete_user(request).await {
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

pub(crate) async fn inner_create_topic(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match CreateTopicRequest::decode(request.as_ref()) {
        Ok(request) => match client.create_topic(request).await {
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

pub(crate) async fn inner_list_topic(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match ListTopicRequest::decode(request.as_ref()) {
        Ok(request) => match client.list_topic(request).await {
            Ok(result) => {
                return Ok(ListTopicReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_delete_topic(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match DeleteTopicRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete_topic(request).await {
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
