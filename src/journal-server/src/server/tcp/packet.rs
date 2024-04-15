use protocol::journal_server::codec::StorageEnginePacket;

#[derive(Debug, Clone)]
pub struct RequestPackage {
    pub connection_id: u64,
    pub packet: StorageEnginePacket,
}

impl RequestPackage {
    pub fn new(connection_id: u64, packet: StorageEnginePacket) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResponsePackage {
    pub connection_id: u64,
    pub packet: StorageEnginePacket,
}

impl ResponsePackage {
    pub fn new(connection_id: u64, packet: StorageEnginePacket) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}
