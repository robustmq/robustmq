use crate::{
    core::subscribe_share::{decode_share_info, is_share_sub},
    metadata::subscriber::Subscriber,
    server::MQTTProtocol,
};
use clients::{placement::mqtt::call::placement_get_share_sub, poll::ClientPool};
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError, log::info, tools::now_second,
};
use dashmap::DashMap;
use protocol::{
    mqtt::{Subscribe, SubscribeProperties},
    placement_center::generate::mqtt::GetShareSubRequest,
};
use std::sync::Arc;

pub async fn parse_subscribe(
    protocol: MQTTProtocol,
    client_id: String,
    subscribe: Subscribe,
    subscribe_properties: Option<SubscribeProperties>,
    client_poll: Arc<ClientPool>,
) -> Result<(), RobustMQError> {
   

    return Ok(());
}
