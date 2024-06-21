use std::{net::SocketAddr, sync::atomic::AtomicU64};

use protocol::mqtt::common::MQTTProtocol;
static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct TCPConnection {
    pub connection_id: u64,
    pub protocol: Option<MQTTProtocol>,
    pub addr: SocketAddr,
}

impl TCPConnection {
    pub fn new(addr: SocketAddr) -> Self {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        TCPConnection {
            connection_id,
            protocol: None,
            addr,
        }
    }

    pub fn connection_id(&self) -> u64 {
        return self.connection_id;
    }

    pub fn set_protocol(&mut self, protocol: MQTTProtocol) {
        self.protocol = Some(protocol);
    }

    pub fn is_mqtt3(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == MQTTProtocol::MQTT3;
        }
        return false;
    }

    pub fn is_mqtt4(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == MQTTProtocol::MQTT4;
        }
        return false;
    }

    pub fn is_mqtt5(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == MQTTProtocol::MQTT5;
        }
        return false;
    }
}
