use std::{net::SocketAddr, sync::atomic::AtomicU64};

use common_base::log::error;
use protocol::mqtt::common::MQTTProtocol;
use tokio::sync::broadcast;
static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct TCPConnection {
    pub connection_id: u64,
    pub protocol: Option<MQTTProtocol>,
    pub addr: SocketAddr,
    pub connection_stop_sx: broadcast::Sender<bool>,
}

impl TCPConnection {
    pub fn new(addr: SocketAddr, connection_stop_sx: broadcast::Sender<bool>) -> Self {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        TCPConnection {
            connection_id,
            protocol: None,
            addr,
            connection_stop_sx,
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

    pub fn stop_connection(&self) {
        match self.connection_stop_sx.send(true) {
            Ok(_) => {}
            Err(e) => {
                error(e.to_string());
            }
        }
    }
}
