use serde::{Deserialize, Serialize};

use super::AvailableFlag;

#[derive(Serialize, Deserialize, Default,Clone)]
pub struct Cluster {
    pub session_expiry_interval: Option<u32>,
    pub topic_alias_max: Option<u16>,
    pub max_qos: Option<u8>,
    pub retain_available: AvailableFlag,
    pub wildcard_subscription_available:  AvailableFlag,
    pub max_packet_size: Option<u32>,
    pub subscription_identifiers_available: AvailableFlag,
    pub shared_subscription_available: AvailableFlag,
    pub server_keep_alive: u16,
    pub receive_max: Option<u16>,
}

impl Cluster {
    pub fn receive_max(&self) -> Option<u16> {
        return self.receive_max;
    }

    pub fn max_qos(&self) -> Option<u8> {
        return self.max_qos;
    }
}
