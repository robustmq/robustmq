use crate::core::cache_manager::CacheManager;
use protocol::mqtt::common::{
    ConnAck, ConnAckProperties, ConnectReturnCode, Disconnect, DisconnectProperties,
    DisconnectReasonCode, MQTTPacket, PingResp, PubAck, PubAckProperties, PubAckReason, PubComp,
    PubCompReason, PubRec, PubRecProperties, PubRecReason, PubRel, PubRelReason, SubAck,
    SubAckProperties, SubscribeReasonCode, UnsubAck, UnsubAckProperties, UnsubAckReason,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct MQTTAckBuild {
    metadata_cache: Arc<CacheManager>,
}

impl MQTTAckBuild {
    pub fn new(metadata_cache: Arc<CacheManager>) -> Self {
        return MQTTAckBuild { metadata_cache };
    }

    pub fn pub_ack_fail(&self, reason: PubAckReason, reason_string: Option<String>) -> MQTTPacket {
        let pub_ack = PubAck { pkid: 0, reason };
        let properties = Some(PubAckProperties {
            reason_string,
            user_properties: Vec::new(),
        });
        return MQTTPacket::PubAck(pub_ack, properties);
    }
    pub fn pub_ack(
        &self,
        pkid: u16,
        reason_string: Option<String>,
        user_properties: Vec<(String, String)>,
    ) -> MQTTPacket {
        let pub_ack = PubAck {
            pkid: pkid,
            reason: PubAckReason::Success,
        };
        let properties = Some(PubAckProperties {
            reason_string: reason_string,
            user_properties: user_properties,
        });
        return MQTTPacket::PubAck(pub_ack, properties);
    }

    pub fn pub_rec(&self, pkid: u16, user_properties: Vec<(String, String)>) -> MQTTPacket {
        let pub_rec = PubRec {
            pkid,
            reason: PubRecReason::Success,
        };

        let properties = PubRecProperties {
            reason_string: Some("".to_string()),
            user_properties,
        };
        return MQTTPacket::PubRec(pub_rec, Some(properties));
    }

    pub fn pub_rel(&self, pkid: u16, reason: PubRelReason) -> MQTTPacket {
        let pub_rel = PubRel { pkid, reason };
        return MQTTPacket::PubRel(pub_rel, None);
    }

    pub fn ping_resp(&self) -> MQTTPacket {
        return MQTTPacket::PingResp(PingResp {});
    }

    pub fn sub_ack(
        &self,
        pkid: u16,
        return_codes: Vec<SubscribeReasonCode>,
        reason_string: Option<String>,
    ) -> MQTTPacket {
        let sub_ack = SubAck { pkid, return_codes };
        let properties = SubAckProperties {
            reason_string,
            user_properties: Vec::new(),
        };
        return MQTTPacket::SubAck(sub_ack, Some(properties));
    }

    pub fn unsub_ack(
        &self,
        pkid: u16,
        reason_string: Option<String>,
        user_properties: Vec<(String, String)>,
    ) -> MQTTPacket {
        let unsub_ack = UnsubAck {
            pkid: pkid,
            reasons: vec![UnsubAckReason::Success],
        };
        let properties = Some(UnsubAckProperties {
            reason_string,
            user_properties,
        });
        return MQTTPacket::UnsubAck(unsub_ack, None);
    }

    pub fn distinct(
        &self,
        reason_code: DisconnectReasonCode,
        reason_string: Option<String>,
    ) -> MQTTPacket {
        let disconnect = Disconnect { reason_code };
        if !reason_string.is_none() {
            let properties = DisconnectProperties {
                session_expiry_interval: None,
                reason_string: reason_string,
                user_properties: Vec::new(),
                server_reference: None,
            };
            return MQTTPacket::Disconnect(disconnect, Some(properties));
        }
        return MQTTPacket::Disconnect(disconnect, None);
    }
}

pub fn publish_comp_fail(pkid: u16) -> MQTTPacket {
    let pub_comp = PubComp {
        pkid,
        reason: PubCompReason::PacketIdentifierNotFound,
    };
    return MQTTPacket::PubComp(pub_comp, None);
}

pub fn publish_comp_success(pkid: u16) -> MQTTPacket {
    let pub_comp = PubComp {
        pkid,
        reason: PubCompReason::Success,
    };
    return MQTTPacket::PubComp(pub_comp, None);
}

