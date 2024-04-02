use crate::server::tcp::packet::RequestPackage;
use crate::{metadata::cache::MetadataCache, server::MQTTProtocol};
use protocol::mqtt::MQTTPacket;
use std::sync::{Arc, RwLock};

use super::packet::MQTTAckBuild;

#[derive(Clone)]
pub struct Command {
    protocol: MQTTProtocol,
    metadata_cache: Arc<RwLock<MetadataCache>>,
    ack_build: MQTTAckBuild,
}

impl Command {
    pub fn new(protocol: MQTTProtocol, metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        let ack_build = MQTTAckBuild::new(protocol.clone(), metadata_cache.clone());
        return Command {
            protocol,
            metadata_cache,
            ack_build,
        };
    }

    pub fn apply(&self, data: RequestPackage) -> MQTTPacket {
        let connect_id = data.connection_id;
        let packet = data.packet;
        match packet {
            MQTTPacket::Connect(connect, properties, last_will, last_will_peoperties, login) => {}
            MQTTPacket::Publish(publish, publish_properties) => {}
            MQTTPacket::Subscribe(subscribe, subscribe_properties) => {}
            MQTTPacket::PingReq(ping) => {}
            MQTTPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {}
            _ => {
                
            }
        }
        return self.ack_build.conn_ack();
    }
}
