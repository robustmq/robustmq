use protocol::mqtt::{QoS, Subscribe, SubscribeProperties};
use serde::{Deserialize, Serialize};

use crate::server::MQTTProtocol;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub protocol: MQTTProtocol,
    pub client_id: String,
    pub packet_identifier: u16,
    pub qos: QoS,
    pub nolocal: bool,
    pub preserve_retain: bool,
    pub subscription_identifier: Option<usize>,
    pub user_properties: Vec<(String, String)>,
    pub is_share_sub: bool,
    pub group_name: Option<String>,
}

#[derive(Clone)]
pub struct SubscribeData {
    pub protocol: MQTTProtocol,
    pub subscribe: Subscribe,
    pub subscribe_properties: Option<SubscribeProperties>,
}
