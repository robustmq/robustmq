use crate::core::cache_manager::CacheManager;
use protocol::mqtt::common::{
    Connect, Disconnect, LastWill, Login, MQTTPacket, PingReq, PubAck, Publish, Subscribe,
    Unsubscribe,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Mqtt3Service {
    cache_manager: Arc<CacheManager>,
    login: bool,
}

impl Mqtt3Service {
    pub fn new(metadata_cache: Arc<CacheManager>) -> Self {
        return Mqtt3Service {
            cache_manager: metadata_cache,
            login: false,
        };
    }

    pub async fn connect(
        &mut self,
        connnect: Connect,
        last_will: Option<LastWill>,
        login: Option<Login>,
    ) -> Option<MQTTPacket> {
        return None;
    }

    pub fn disconnect(&self, disconnect: Disconnect) -> Option<MQTTPacket> {
        return None;
    }

    pub fn publish(&self, publish: Publish) -> Option<MQTTPacket> {
        return None;
    }

    pub fn publish_ack(&self, pub_ack: PubAck) {}

    pub fn subscribe(&self, subscribe: Subscribe) -> Option<MQTTPacket> {
        return None;
    }

    pub fn ping(&self, ping: PingReq) -> Option<MQTTPacket> {
        return None;
    }

    pub fn un_subscribe(&self, un_subscribe: Unsubscribe) -> Option<MQTTPacket> {
        return None;
    }

    fn un_login_err(&self) -> Option<MQTTPacket> {
        return None;
    }
}
