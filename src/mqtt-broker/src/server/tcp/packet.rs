use std::net::SocketAddr;

use protocol::mqtt::common::MQTTPacket;

#[derive(Clone, Debug)]
pub struct RequestPackage {
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub packet: MQTTPacket,
}

impl RequestPackage {
    pub fn new(connection_id: u64, addr: SocketAddr, packet: MQTTPacket) -> Self {
        Self {
            connection_id,
            addr,
            packet,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResponsePackage {
    pub connection_id: u64,
    pub packet: MQTTPacket,
}

impl ResponsePackage {
    pub fn new(connection_id: u64, packet: MQTTPacket) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}
