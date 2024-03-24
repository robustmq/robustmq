use protocol::mqtt::Packet;

#[derive(Clone,PartialEq)]
pub enum MqttProtocol {
    Mqtt4,
    Mqtt5,
}

#[derive(Debug)]
pub struct RequestPackage {
    pub connection_id: u64,
    pub packet: Packet,
}

impl RequestPackage {
    pub fn new(connection_id: u64, packet: Packet) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}

#[derive(Debug)]
pub struct ResponsePackage {
    pub connection_id: u64,
    pub packet: Packet,
}

impl ResponsePackage {
    pub fn new(connection_id: u64, packet: Packet) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}
