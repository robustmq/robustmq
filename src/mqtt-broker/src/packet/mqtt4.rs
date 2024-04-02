use protocol::mqtt::{
    Connect, LastWill, Login, MQTTPacket, PingReq, Publish, PublishProperties, Subscribe,
    Unsubscribe, UnsubscribeProperties,
};

use crate::metadata::cache::MetadataCache;
use std::sync::{Arc, RwLock};

use super::packet::MQTTAckBuild;
#[derive(Clone)]
pub struct Mqtt4Service {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    ack_build: MQTTAckBuild,
}

impl Mqtt4Service {
    pub fn new(metadata_cache: Arc<RwLock<MetadataCache>>, ack_build: MQTTAckBuild) -> Self {
        return Mqtt4Service {
            metadata_cache,
            ack_build,
        };
    }

    pub fn connect(
        &self,
        connnect: Connect,
        last_will: Option<LastWill>,
        login: Option<Login>,
    ) -> MQTTPacket {
        return self.ack_build.conn_ack();
    }

    pub fn publish(&self, publish: Publish) -> MQTTPacket {
        return self.ack_build.pub_ack();
    }

    pub fn subscribe(&self, subscribe: Subscribe) -> MQTTPacket {
        return self.ack_build.sub_ack();
    }

    pub fn ping(&self, ping: PingReq) -> MQTTPacket {
        return self.ack_build.ping_resp();
    }

    pub fn un_subscribe(&self, un_subscribe: Unsubscribe) -> MQTTPacket {
        return self.ack_build.unsub_ack();
    }
}
