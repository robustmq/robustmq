use crate::{metadata::cache::MetadataCache, server::MQTTProtocol};
use protocol::mqtt::{
    ConnAck, ConnAckProperties, ConnectReturnCode, Disconnect, DisconnectReasonCode, MQTTPacket,
    PingResp, PubAck, PubAckProperties, PubAckReason, SubAck, SubAckProperties,
    SubscribeReasonCode, UnsubAck, UnsubAckProperties, UnsubAckReason,
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct MQTTAckBuild {
    protocol: MQTTProtocol,
    metadata_cache: Arc<RwLock<MetadataCache>>,
}

impl MQTTAckBuild {
    pub fn new(protocol: MQTTProtocol, metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        return MQTTAckBuild {
            protocol,
            metadata_cache,
        };
    }

    pub fn conn_ack(
        &self,
        client_id: String,
        auto_client_id: bool,
        reason_string: Option<String>,
        user_properties: Vec<(String, String)>,
        response_information: Option<String>,
        server_reference: Option<String>,
    ) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: false,
            code: ConnectReturnCode::Success,
        };
        let cache = self.metadata_cache.read().unwrap();
        let cluster = cache.cluster_info.clone();
        if let Some(session) = cache.session_info.get(&client_id) {
            let assigned_client_identifier = if auto_client_id {
                None
            } else {
                Some(client_id)
            };

            let ack_properties = ConnAckProperties {
                session_expiry_interval: session.session_expiry_interval,
                receive_max: cluster.receive_max(),
                max_qos: cluster.max_qos(),
                retain_available: Some(cluster.retain_available()),
                max_packet_size: Some(cluster.max_packet_size()),
                assigned_client_identifier: assigned_client_identifier,
                topic_alias_max: Some(cluster.topic_alias_max()),
                reason_string: reason_string,
                user_properties: user_properties,
                wildcard_subscription_available: Some(cluster.wildcard_subscription_available()),
                subscription_identifiers_available: Some(
                    cluster.subscription_identifiers_available(),
                ),
                shared_subscription_available: Some(cluster.shared_subscription_available()),
                server_keep_alive: Some(cluster.server_keep_alive()),
                response_information: response_information,
                server_reference: server_reference,
                authentication_method: None,
                authentication_data: None,
            };
            return MQTTPacket::ConnAck(conn_ack, Some(ack_properties));
        }
        return self.distinct(DisconnectReasonCode::UnspecifiedError);
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

    pub fn pub_rec(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
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

    pub fn sub_ack(
        &self,
        pkid: u16,
        reason_string: Option<String>,
        user_properties: Vec<(String, String)>,
    ) -> MQTTPacket {
        let sub_ack = SubAck {
            pkid: pkid,
            return_codes: vec![SubscribeReasonCode::QoS0],
        };

        let sub_properties = Some(SubAckProperties {
            reason_string,
            user_properties,
        });
        return MQTTPacket::SubAck(sub_ack, sub_properties);
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

    pub fn distinct(&self, reason_code: DisconnectReasonCode) -> MQTTPacket {
        let disconnect = Disconnect { reason_code };
        return MQTTPacket::Disconnect(disconnect, None);
    }
}
