use protocol::storage_engine::codec::StorageEnginePacket;

pub struct Command {
    packet: StorageEnginePacket,
}

impl Command {
    pub fn new(packet: StorageEnginePacket) -> Self {
        return Command { packet };
    }

    pub fn apply(&self) {
        
    }
}
