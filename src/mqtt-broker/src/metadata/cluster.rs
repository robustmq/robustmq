use serde::{Deserialize, Serialize};

use super::AvailableFlag;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Cluster {
    pub session_expiry_interval: u32,
    pub topic_alias_max: u16,
    pub max_qos: Option<u8>,
    pub retain_available: AvailableFlag,
    pub wildcard_subscription_available: AvailableFlag,
    pub max_packet_size: u32,
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

    pub fn server_keep_alive(&self) -> u16 {
        return self.server_keep_alive;
    }

    pub fn retain_available(&self) -> u8 {
        return self.retain_available.clone() as u8;
    }

    pub fn max_packet_size(&self) -> u32 {
        return self.max_packet_size;
    }

    pub fn topic_alias_max(&self) -> u16 {
        return self.topic_alias_max;
    }

    pub fn wildcard_subscription_available(&self) -> u8{
        return self.wildcard_subscription_available.clone() as u8;
    }

    pub fn subscription_identifiers_available(&self) -> u8{
        return self.subscription_identifiers_available.clone() as u8;
    }

    pub fn shared_subscription_available(&self) -> u8{
        return self.shared_subscription_available.clone() as u8;
    }
}
