use protocol::mqtt::{Filter, QoS, SubscribeProperties};
use serde::{Deserialize, Serialize};

use crate::server::MQTTProtocol;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub protocol: MQTTProtocol,
    pub client_id: String,
    pub sub_path: String,
    pub topic_name: String,
    pub group_name: Option<String>,
    pub topic_id: String,
    pub qos: QoS,
    pub nolocal: bool,
    pub preserve_retain: bool,
    pub is_contain_rewrite_flag: bool,
    pub subscription_identifier: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubscribeData {
    pub protocol: MQTTProtocol,
    pub filter: Filter,
    pub subscribe_properties: Option<SubscribeProperties>,
}
