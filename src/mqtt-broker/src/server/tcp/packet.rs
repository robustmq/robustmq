use protocol::mqtt::MQTTPacket;
#[derive(Clone, PartialEq)]

#[derive(Debug)]
pub struct RequestPackage {
    pub connection_id: u64,
    pub packet: MQTTPacket,
}

impl RequestPackage {
    pub fn new(connection_id: u64, packet: MQTTPacket) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}

#[derive(Debug)]
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
