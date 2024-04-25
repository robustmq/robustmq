use crate::{
    metadata::{cache::MetadataCache, cluster::Cluster, session::Session},
    server::MQTTProtocol,
};
use protocol::mqtt::{
    ConnAck, ConnAckProperties, ConnectReturnCode, Disconnect, DisconnectProperties,
    DisconnectReasonCode, MQTTPacket, PingResp, PubAck, PubAckProperties, PubAckReason, SubAck,
    SubAckProperties, SubscribeReasonCode, UnsubAck, UnsubAckProperties, UnsubAckReason,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct MQTTAckBuild<T> {
    protocol: MQTTProtocol,
    metadata_cache: Arc<MetadataCache<T>>,
}

impl<T> MQTTAckBuild<T> {
    pub fn new(protocol: MQTTProtocol, metadata_cache: Arc<MetadataCache<T>>) -> Self {
        return MQTTAckBuild {
            protocol,
            metadata_cache,
        };
    }

    pub async fn conn_ack(
        &self,
        cluster: &Cluster,
        session: &Session,
        client_id: String,
        auto_client_id: bool,
    ) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: session.session_present,
            code: ConnectReturnCode::Success,
        };

        let assigned_client_identifier = if auto_client_id {
            Some(client_id)
        } else {
            None
        };

        let ack_properties = ConnAckProperties {
            session_expiry_interval: session.session_expiry_interval,
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
        return MQTTPacket::ConnAck(conn_ack, Some(ack_properties));
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

    pub fn pub_rec(&self,session_present:bool) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present,
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
