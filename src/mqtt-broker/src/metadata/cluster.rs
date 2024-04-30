use protocol::mqtt::QoS;
use serde::{Deserialize, Serialize};

use super::AvailableFlag;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Cluster {
    pub session_expiry_interval: u32,
    pub topic_alias_max: u16,
    pub max_qos: QoS,
    pub retain_available: AvailableFlag,
    pub wildcard_subscription_available: AvailableFlag,
    pub max_packet_size: u32,
    pub subscription_identifiers_available: AvailableFlag,
    pub shared_subscription_available: AvailableFlag,
    pub server_keep_alive: u16,
    pub receive_max: u16,
    pub secret_free_login: bool,
}

impl Cluster {
    pub fn new() -> Self {
        return Cluster::default();
    }
    pub fn receive_max(&self) -> u16 {
        return self.receive_max;
    }

    pub fn max_qos(&self) -> u8 {
        return self.max_qos.into();
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

    pub fn wildcard_subscription_available(&self) -> u8 {
        return self.wildcard_subscription_available.clone() as u8;
    }

    pub fn subscription_identifiers_available(&self) -> u8 {
        return self.subscription_identifiers_available.clone() as u8;
    }

    pub fn shared_subscription_available(&self) -> u8 {
        return self.shared_subscription_available.clone() as u8;
    }

    pub fn secret_free_login(&self) -> bool {
        return self.secret_free_login;
    }
}
