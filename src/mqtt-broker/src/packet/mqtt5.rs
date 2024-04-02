use std::sync::{Arc, RwLock};

use protocol::mqtt::{
    Connect, ConnectProperties, LastWill, LastWillProperties, Login, MQTTPacket, PingReq, Publish,
    PublishProperties, Subscribe, SubscribeProperties, Unsubscribe, UnsubscribeProperties,
};

use crate::metadata::cache::MetadataCache;

use super::packet::MQTTAckBuild;
#[derive(Clone)]
pub struct Mqtt5Service {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    ack_build: MQTTAckBuild,
}

impl Mqtt5Service {
    pub fn new(metadata_cache: Arc<RwLock<MetadataCache>>, ack_build: MQTTAckBuild) -> Self {
        return Mqtt5Service {
            metadata_cache,
            ack_build,
        };
    }
    pub fn connect(
        &self,
        connnect: Connect,
        properties: Option<ConnectProperties>,
        last_will: Option<LastWill>,
        last_will_properties: Option<LastWillProperties>,
        login: Option<Login>,
    ) -> MQTTPacket {
        return self.ack_build.conn_ack();
    }

    pub fn publish(
        &self,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> MQTTPacket {
        return self.ack_build.pub_ack();
    }

    pub fn subscribe(
        &self,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) -> MQTTPacket {
        return self.ack_build.sub_ack();
    }

    pub fn ping(&self, ping: PingReq) -> MQTTPacket {
        return self.ack_build.ping_resp();
    }

    pub fn un_subscribe(
        &self,
        un_subscribe: Unsubscribe,
        un_subscribe_properties: Option<UnsubscribeProperties>,
    ) -> MQTTPacket {
        return self.ack_build.unsub_ack();
    }
}
