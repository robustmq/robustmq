use common_base::errors::RobustMQError;
use prost::Message;
use protocol::broker_server::generate::mqtt::{
    mqtt_broker_service_client::MqttBrokerServiceClient, CommonReply, DeleteSessionRequest,
    UpdateCacheRequest,
};
use tonic::transport::Channel;

pub(crate) async fn inner_delete_session(
    mut client: MqttBrokerServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match DeleteSessionRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete_session(request).await {
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

pub(crate) async fn inner_update_cache(
    mut client: MqttBrokerServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match UpdateCacheRequest::decode(request.as_ref()) {
        Ok(request) => match client.update_cache(request).await {
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
