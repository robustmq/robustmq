use protocol::mqtt::common::QoS;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MQTTCluster {
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
    pub max_message_expiry_interval: u64,
    pub default_message_expiry_interval: u64,
    pub client_pkid_persistent: bool,
    pub is_self_protection_status: bool,
}

impl MQTTCluster {
    pub fn new() -> Self {
        return MQTTCluster {
            session_expiry_interval: 1800,
            topic_alias_max: 65535,
            max_qos: QoS::ExactlyOnce,
            retain_available: AvailableFlag::Enable,
            max_packet_size: 1024 * 1024 * 10,
            wildcard_subscription_available: AvailableFlag::Enable,
            subscription_identifiers_available: AvailableFlag::Enable,
            shared_subscription_available: AvailableFlag::Enable,
            server_keep_alive: 60,
            receive_max: 65535,
            secret_free_login: false,
            max_message_expiry_interval: 315360000,
            default_message_expiry_interval: 3600,
            client_pkid_persistent: false,
            is_self_protection_status: false,
        };
    }
    pub fn receive_max(&self) -> u16 {
        return self.receive_max;
    }

    pub fn max_qos(&self) -> QoS {
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

    pub fn wildcard_subscription_available(&self) -> u8 {
        return self.wildcard_subscription_available.clone() as u8;
    }

    pub fn subscription_identifiers_available(&self) -> u8 {
        return self.subscription_identifiers_available.clone() as u8;
    }

    pub fn shared_subscription_available(&self) -> u8 {
        return self.shared_subscription_available.clone() as u8;
    }

    pub fn is_secret_free_login(&self) -> bool {
        return self.secret_free_login;
    }

    pub fn encode(&self) -> Vec<u8> {
        return serde_json::to_vec(&self).unwrap();
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub enum AvailableFlag {
    #[default]
    Enable,
    Disable,
}

impl From<AvailableFlag> for u8 {
    fn from(flag: AvailableFlag) -> Self {
        match flag {
            AvailableFlag::Enable => 1,
            AvailableFlag::Disable => 0,
        }
    }
}

pub enum Available {
    Enable,
    Disable,
}

pub fn available_flag(flag: Available) -> AvailableFlag {
    match flag {
        Available::Enable => return AvailableFlag::Enable,
        Available::Disable => return AvailableFlag::Disable,
    }
}
