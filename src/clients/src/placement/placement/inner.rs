use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    placement::{
        placement_center_service_client::PlacementCenterServiceClient, HeartbeatRequest,
        RegisterNodeRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest,
        SendRaftMessageReply, SendRaftMessageRequest, UnRegisterNodeRequest,
    },
};
use tonic::transport::Channel;

pub(crate) async fn inner_register_node(
    mut client: PlacementCenterServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match RegisterNodeRequest::decode(request.as_ref()) {
        Ok(request) => match client.register_node(request).await {
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

pub(crate) async fn inner_unregister_node(
    mut client: PlacementCenterServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match UnRegisterNodeRequest::decode(request.as_ref()) {
        Ok(request) => match client.un_register_node(request).await {
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

pub(crate) async fn inner_heartbeat(
    mut client: PlacementCenterServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match HeartbeatRequest::decode(request.as_ref()) {
        Ok(request) => match client.heartbeat(request).await {
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

pub(crate) async fn inner_send_raft_message(
    mut client: PlacementCenterServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match SendRaftMessageRequest::decode(request.as_ref()) {
        Ok(request) => match client.send_raft_message(request).await {
            Ok(result) => {
                return Ok(SendRaftMessageReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_send_raft_conf_change(
    mut client: PlacementCenterServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match SendRaftConfChangeRequest::decode(request.as_ref()) {
        Ok(request) => match client.send_raft_conf_change(request).await {
            Ok(result) => {
                return Ok(SendRaftConfChangeReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}
