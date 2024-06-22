use super::packet::MQTTAckBuild;
use crate::core::cache_manager::CacheManager;
use common_base::tools::unique_id;
use protocol::mqtt::common::{
    ConnAck, Connect, ConnectReturnCode, Disconnect, DisconnectReasonCode, LastWill, Login,
    MQTTPacket, PingReq, PubAck, PubAckReason, Publish, Subscribe, Unsubscribe,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Mqtt4Service {
    cache_manager: Arc<CacheManager>,
    ack_build: MQTTAckBuild,
    login: bool,
}

impl Mqtt4Service {
    pub fn new(cache_manager: Arc<CacheManager>, ack_build: MQTTAckBuild) -> Self {
        return Mqtt4Service {
            cache_manager,
            ack_build,
            login: false,
        };
    }

    pub async fn connect(
        &mut self,
        connnect: Connect,
        last_will: Option<LastWill>,
        login: Option<Login>,
    ) -> MQTTPacket {
        self.login = true;
        let client_id = unique_id();
        let auto_client_id = true;
        let reason_string = Some("".to_string());
        let response_information = Some("".to_string());
        let server_reference = Some("".to_string());

        return build_connack_success(true);
    }

    pub fn disconnect(&self, disconnect: Disconnect) -> MQTTPacket {
        return build_distinct(DisconnectReasonCode::NormalDisconnection);
    }
    pub fn publish(&self, publish: Publish) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self.ack_build.pub_ack(0, None, Vec::new());
    }

    pub fn publish_ack(&self, pub_ack: PubAck) {}

    pub fn subscribe(&self, subscribe: Subscribe) -> MQTTPacket {
        if self.login {
            return self.un_login_err();
        }
        return self.ack_build.sub_ack(0, Vec::new());
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

    fn un_login_err(&self) -> MQTTPacket {
        return self.ack_build.distinct(
            protocol::mqtt::common::DisconnectReasonCode::NotAuthorized,
            None,
        );
    }
}

fn build_connack_success(session_present: bool) -> MQTTPacket {
    let conn_ack = ConnAck {
        session_present,
        code: ConnectReturnCode::Success,
    };
    return MQTTPacket::ConnAck(conn_ack, None);
}

fn build_distinct(reason_code: DisconnectReasonCode) -> MQTTPacket {
    let disconnect = Disconnect { reason_code };
    return MQTTPacket::Disconnect(disconnect, None);
}
