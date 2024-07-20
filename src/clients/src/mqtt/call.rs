use crate::poll::ClientPool;
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::broker_server::generate::mqtt::{
    CommonReply, DeleteSessionRequest, SendLastWillMessageRequest, UpdateCacheRequest
};
use std::sync::Arc;

use super::{retry_call, MQTTBrokerInterface, MQTTBrokerService};

pub async fn broker_mqtt_delete_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteSessionRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteSessionRequest::encode_to_vec(&request);
    match retry_call(
        MQTTBrokerService::Mqtt,
        MQTTBrokerInterface::DeleteSession,
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

pub async fn broker_mqtt_update_cache(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UpdateCacheRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = UpdateCacheRequest::encode_to_vec(&request);
    match retry_call(
        MQTTBrokerService::Mqtt,
        MQTTBrokerInterface::UpdateCache,
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


pub async fn send_last_will_message(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendLastWillMessageRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = SendLastWillMessageRequest::encode_to_vec(&request);
    match retry_call(
        MQTTBrokerService::Mqtt,
        MQTTBrokerInterface::UpdateCache,
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
