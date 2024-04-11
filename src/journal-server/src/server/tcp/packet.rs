use protocol::storage_engine::codec::StorageEnginePacket;

#[derive(Debug)]
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

#[derive(Debug)]
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
