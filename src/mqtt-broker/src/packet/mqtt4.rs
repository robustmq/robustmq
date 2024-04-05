use common_base::tools::unique_id_string;
use protocol::mqtt::{
    Connect, Disconnect, DisconnectReasonCode, LastWill, Login, MQTTPacket, PingReq, Publish,
    Subscribe, Unsubscribe,
};

use crate::metadata::{cache::MetadataCache, hearbeat::HeartbeatManager};
use std::sync::{Arc, RwLock};

use super::packet::MQTTAckBuild;
#[derive(Clone)]
pub struct Mqtt4Service {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    ack_build: MQTTAckBuild,
    login: bool,
    heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
}

impl Mqtt4Service {
    pub fn new(
        metadata_cache: Arc<RwLock<MetadataCache>>,
        ack_build: MQTTAckBuild,
        heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
    ) -> Self {
        return Mqtt4Service {
            metadata_cache,
            ack_build,
            login: false,
            heartbeat_manager,
        };
    }

    pub fn connect(
        &mut self,
        connnect: Connect,
        last_will: Option<LastWill>,
        login: Option<Login>,
    ) -> MQTTPacket {
        self.login = true;
        let client_id = unique_id_string();
        let auto_client_id = true;
        let reason_string = Some("".to_string());
        let user_properties = Vec::new();
        let response_information = Some("".to_string());
        let server_reference = Some("".to_string());
        return self.ack_build.conn_ack(
            client_id,
            auto_client_id,
            reason_string,
            user_properties,
            response_information,
            server_reference,
        );
    }

    pub fn publish(&self, publish: Publish) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self.ack_build.pub_ack(0, None, Vec::new());
    }

    pub fn subscribe(&self, subscribe: Subscribe) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self.ack_build.sub_ack(0, None, Vec::new());
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
        return self.ack_build.unsub_ack(0, None, Vec::new());
    }

    pub fn disconnect(&self, disconnect: Disconnect) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self
            .ack_build
            .distinct(DisconnectReasonCode::NormalDisconnection);
    }

    fn un_login_err(&self) -> MQTTPacket {
        return self
            .ack_build
            .distinct(protocol::mqtt::DisconnectReasonCode::NotAuthorized);
    }
}
