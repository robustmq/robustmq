use protocol::mqtt::QoS;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub connect_id: u64,
    pub packet_identifier: u16,
    pub qos: QoS,
    pub nolocal: bool,
    pub preserve_retain: bool,
    pub subscription_identifier: Option<usize>,
    pub user_properties: Vec<(String, String)>,
}