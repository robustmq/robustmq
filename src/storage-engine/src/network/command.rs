use std::fmt::format;

use common_base::log::error_engine;
use protocol::storage_engine::codec::StorageEnginePacket;

use super::{response::build_produce_resp, services::Services};

pub struct Command {
    packet: StorageEnginePacket,
    services: Services,
}

impl Command {
    pub fn new(packet: StorageEnginePacket, services: Services) -> Self {
        return Command { packet, services };
    }

    pub fn apply(&self) -> StorageEnginePacket {
        match self.packet.clone() {
            StorageEnginePacket::ProduceReq(data) => {
                self.services.produce();
            }
            StorageEnginePacket::FetchReq(data) => {
                self.services.fetch();
            }
            _ => {
                error_engine(format!(
                    "server received an unrecognized request, request info: {:?}",
                    self.packet
                ));
            }
        }
        return build_produce_resp();
    }
}
