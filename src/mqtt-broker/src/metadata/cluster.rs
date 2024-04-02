use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Cluster {
    pub topic_alias_max: Option<u16>,
    pub max_qos: Option<u8>,
    pub retain_available:  Option<u8>,
    pub max_packet_size: Option<u32>,
    pub wildcard_subscription_available: Option<u8>,
    pub subscription_identifiers_available:  Option<u8>,
    pub shared_subscription_available: Option<u8>,
    pub server_keep_alive: Option<u16>,
}
