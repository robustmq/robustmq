use crate::{
    metadata::{cache::MetadataCacheManager, cluster::Cluster, session::Session},
    server::MQTTProtocol,
};
use protocol::mqtt::{
    ConnAck, ConnAckProperties, ConnectReturnCode, Disconnect, DisconnectProperties,
    DisconnectReasonCode, MQTTPacket, PingResp, PubAck, PubAckProperties, PubAckReason, PubComp,
    PubCompReason, PubRec, PubRecProperties, PubRecReason, SubAck, SubAckProperties,
    SubscribeReasonCode, UnsubAck, UnsubAckProperties, UnsubAckReason,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct MQTTAckBuild<T> {
    protocol: MQTTProtocol,
    metadata_cache: Arc<MetadataCacheManager<T>>,
}

impl<T> MQTTAckBuild<T> {
    pub fn new(protocol: MQTTProtocol, metadata_cache: Arc<MetadataCacheManager<T>>) -> Self {
        return MQTTAckBuild {
            protocol,
            metadata_cache,
        };
    }

    pub fn packet_connect_success(
        &self,
        cluster: &Cluster,
        client_id: String,
        auto_client_id: bool,
        session_expiry_interval: u32,
        session_present: bool,
    ) -> MQTTPacket {
        let assigned_client_identifier = if auto_client_id {
            Some(client_id)
        } else {
            None
        };

        let properties = ConnAckProperties {
            session_expiry_interval: Some(session_expiry_interval),
            receive_max: cluster.receive_max(),
            max_qos: cluster.max_qos(),
            retain_available: Some(cluster.retain_available()),
            max_packet_size: Some(cluster.max_packet_size()),
            assigned_client_identifier: assigned_client_identifier,
            topic_alias_max: Some(cluster.topic_alias_max()),
            reason_string: None,
            user_properties: Vec::new(),
            wildcard_subscription_available: Some(cluster.wildcard_subscription_available()),
            subscription_identifiers_available: Some(cluster.subscription_identifiers_available()),
            shared_subscription_available: Some(cluster.shared_subscription_available()),
            server_keep_alive: Some(cluster.server_keep_alive()),
            response_information: None,
            server_reference: None,
            authentication_method: None,
            authentication_data: None,
        };
        return MQTTPacket::ConnAck(
            ConnAck {
                session_present,
                code: ConnectReturnCode::Success,
            },
            Some(properties),
        );
    }

    pub fn packet_connect_fail(
        &self,
        code: ConnectReturnCode,
        reason_string: Option<String>,
    ) -> MQTTPacket {
        let mut properties = ConnAckProperties::default();
        properties.reason_string = reason_string;
        return MQTTPacket::ConnAck(
            ConnAck {
                session_present: false,
                code,
            },
            Some(properties),
        );
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

    pub fn pub_rel(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn pub_comp(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn ping_resp(&self) -> MQTTPacket {
        return MQTTPacket::PingResp(PingResp {});
    }

    pub fn sub_ack(&self, pkid: u16) -> MQTTPacket {
        let sub_ack = SubAck {
            pkid: pkid,
            return_codes: vec![SubscribeReasonCode::QoS0],
        };
        return MQTTPacket::SubAck(sub_ack, None);
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
