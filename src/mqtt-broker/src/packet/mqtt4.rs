use protocol::mqtt::{
    Connect, LastWill, Login, MQTTPacket, PingReq, Publish, Subscribe, Unsubscribe,
};

use crate::metadata::cache::MetadataCache;
use std::sync::{Arc, RwLock};

use super::packet::MQTTAckBuild;
#[derive(Clone)]
pub struct Mqtt4Service {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    ack_build: MQTTAckBuild,
    login: bool,
}

impl Mqtt4Service {
    pub fn new(metadata_cache: Arc<RwLock<MetadataCache>>, ack_build: MQTTAckBuild) -> Self {
        return Mqtt4Service {
            metadata_cache,
            ack_build,
            login: false,
        };
    }

    pub fn connect(
        &mut self,
        connnect: Connect,
        last_will: Option<LastWill>,
        login: Option<Login>,
    ) -> MQTTPacket {
        self.login = true;
        return self.ack_build.conn_ack();
    }

    pub fn publish(&self, publish: Publish) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self.ack_build.pub_ack();
    }

    pub fn subscribe(&self, subscribe: Subscribe) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self.ack_build.sub_ack();
    }

    pub fn ping(&self, ping: PingReq) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self.ack_build.ping_resp();
    }

    pub fn un_subscribe(&self, un_subscribe: Unsubscribe) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self.ack_build.unsub_ack();
    }

    fn un_login_err(&self) -> MQTTPacket {
        return self
            .ack_build
            .distinct(protocol::mqtt::DisconnectReasonCode::NotAuthorized);
    }
}
