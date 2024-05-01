use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        mqtt_service_client::MqttServiceClient, DeleteShareSubRequest, GetShareSubReply,
        GetShareSubRequest,
    },
};
use tonic::transport::Channel;

pub(crate) async fn inner_get_share_sub(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match GetShareSubRequest::decode(request.as_ref()) {
        Ok(request) => match client.get_share_sub(request).await {
            Ok(result) => {
                return Ok(GetShareSubReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(RobustMQError::MetaGrpcStatus(e)),
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_delete_share_sub(
    mut client: MqttServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match DeleteShareSubRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete_share_sub(request).await {
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
