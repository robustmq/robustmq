use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        mqtt_service_client::MqttServiceClient, GetShareSubLeaderReply,
        GetShareSubLeaderRequest,
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
