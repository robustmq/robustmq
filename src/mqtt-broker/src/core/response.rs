// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::connection::response_information;
use common_config::config::BrokerConfig;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    ConnAck, ConnAckProperties, ConnectProperties, ConnectReturnCode, Disconnect,
    DisconnectProperties, DisconnectReasonCode, MqttPacket, MqttProtocol, PingResp, PubAck,
    PubAckProperties, PubAckReason, PubComp, PubCompProperties, PubCompReason, PubRec,
    PubRecProperties, PubRecReason, PubRel, PubRelProperties, PubRelReason, SubAck,
    SubAckProperties, SubscribeReasonCode, UnsubAck, UnsubAckProperties, UnsubAckReason,
};
use tracing::{debug, info};

pub fn build_pub_ack_fail(
    protocol: &MqttProtocol,
    connection: &MQTTConnection,
    pkid: u16,
    reason_string: Option<String>,
    is_pub_ack: bool,
) -> MqttPacket {
    if is_pub_ack {
        return build_puback(
            protocol,
            connection,
            pkid,
            PubAckReason::UnspecifiedError,
            reason_string,
            Vec::new(),
        );
    }

    build_pubrec(
        protocol,
        connection,
        pkid,
        PubRecReason::UnspecifiedError,
        reason_string,
        Vec::new(),
    )
}

#[derive(Clone)]
pub struct ResponsePacketMqttConnectSuccessContext {
    pub protocol: MqttProtocol,
    pub cluster: BrokerConfig,
    pub client_id: String,
    pub auto_client_id: bool,
    pub session_expiry_interval: u32,
    pub session_present: bool,
    pub keep_alive: u16,
    pub connect_properties: Option<ConnectProperties>,
}

pub fn response_packet_mqtt_connect_success(
    context: ResponsePacketMqttConnectSuccessContext,
) -> MqttPacket {
    if !context.protocol.is_mqtt5() {
        return MqttPacket::ConnAck(
            ConnAck {
                session_present: context.session_present,
                code: ConnectReturnCode::Success,
            },
            None,
        );
    }

    let assigned_client_identifier = if context.auto_client_id {
        Some(context.client_id)
    } else {
        None
    };

    let properties = ConnAckProperties {
        session_expiry_interval: Some(context.session_expiry_interval),
        receive_max: Some(context.cluster.mqtt_protocol_config.receive_max),
        max_qos: Some(context.cluster.mqtt_protocol_config.max_qos),
        retain_available: Some(1),
        max_packet_size: Some(context.cluster.mqtt_protocol_config.max_packet_size),
        assigned_client_identifier,
        topic_alias_max: Some(context.cluster.mqtt_protocol_config.topic_alias_max),
        reason_string: None,
        user_properties: Vec::new(),
        wildcard_subscription_available: Some(1),
        subscription_identifiers_available: Some(1),
        shared_subscription_available: Some(1),
        server_keep_alive: Some(context.keep_alive),
        response_information: response_information(&context.connect_properties),
        server_reference: None,
        authentication_method: None,
        authentication_data: None,
    };
    MqttPacket::ConnAck(
        ConnAck {
            session_present: context.session_present,
            code: ConnectReturnCode::Success,
        },
        Some(properties),
    )
}

pub fn response_packet_mqtt_connect_fail(
    protocol: &MqttProtocol,
    code: ConnectReturnCode,
    connect_properties: &Option<ConnectProperties>,
    error_reason: Option<String>,
) -> MqttPacket {
    debug!("{code:?},{error_reason:?}");
    if !protocol.is_mqtt5() {
        let new_code = if code == ConnectReturnCode::ClientIdentifierNotValid {
            ConnectReturnCode::IdentifierRejected
        } else if code == ConnectReturnCode::ProtocolError {
            ConnectReturnCode::UnacceptableProtocolVersion
        } else if code == ConnectReturnCode::Success || code == ConnectReturnCode::NotAuthorized {
            code
        } else {
            ConnectReturnCode::ServiceUnavailable
        };
        return MqttPacket::ConnAck(
            ConnAck {
                session_present: false,
                code: new_code,
            },
            None,
        );
    }
    let mut properties = ConnAckProperties::default();
    // if is_request_problem_info(connect_properties) {
    //     properties.reason_string = error_reason;
    // }
    MqttPacket::ConnAck(
        ConnAck {
            session_present: false,
            code,
        },
        Some(properties),
    )
}

pub fn response_packet_mqtt_distinct(
    protocol: &MqttProtocol,
    code: Option<DisconnectReasonCode>,
    connection: &MQTTConnection,
    reason_string: Option<String>,
) -> MqttPacket {
    if !protocol.is_mqtt5() {
        return MqttPacket::Disconnect(Disconnect { reason_code: None }, None);
    }
    let mut properties = DisconnectProperties::default();
    if connection.is_response_problem_info() {
        properties.reason_string = reason_string;
    }

    MqttPacket::Disconnect(Disconnect { reason_code: code }, None)
}

pub fn response_packet_mqtt_distinct_by_reason(
    protocol: &MqttProtocol,
    code: Option<DisconnectReasonCode>,
    server_reference: Option<String>,
) -> MqttPacket {
    if !protocol.is_mqtt5() {
        return MqttPacket::Disconnect(Disconnect { reason_code: None }, None);
    }

    MqttPacket::Disconnect(
        Disconnect { reason_code: code },
        Some(DisconnectProperties {
            reason_string: Some("".to_string()),
            server_reference,
            ..Default::default()
        }),
    )
}

pub fn build_puback(
    protocol: &MqttProtocol,
    connection: &MQTTConnection,
    pkid: u16,
    reason: PubAckReason,
    reason_string: Option<String>,
    user_properties: Vec<(String, String)>,
) -> MqttPacket {
    if reason != PubAckReason::Success {
        debug!(
            "client_id:{},reason:{reason:?}, reason string: {reason_string:?}",
            connection.client_id
        );
    }

    if protocol.is_mqtt3() || protocol.is_mqtt4() {
        let pub_ack = PubAck { pkid, reason: None };
        return MqttPacket::PubAck(pub_ack, None);
    }

    let pub_ack = PubAck {
        pkid,
        reason: Some(reason),
    };
    let mut properties = PubAckProperties::default();
    if connection.is_response_problem_info() {
        properties.reason_string = reason_string;
    }
    properties.user_properties = user_properties;
    MqttPacket::PubAck(pub_ack, Some(properties))
}

pub fn build_pubrec(
    protocol: &MqttProtocol,
    connection: &MQTTConnection,
    pkid: u16,
    reason: PubRecReason,
    reason_string: Option<String>,
    user_properties: Vec<(String, String)>,
) -> MqttPacket {
    if reason != PubRecReason::Success {
        info!(
            "client_id:{},reason:{reason:?}, reason string: {reason_string:?}",
            connection.client_id
        );
    }

    if protocol.is_mqtt3() || protocol.is_mqtt4() {
        return MqttPacket::PubRec(PubRec { pkid, reason: None }, None);
    }

    let pub_ack = PubRec {
        pkid,
        reason: Some(reason),
    };
    let mut properties = PubRecProperties::default();
    if connection.is_response_problem_info() {
        properties.reason_string = reason_string;
    }
    properties.user_properties = user_properties;
    MqttPacket::PubRec(pub_ack, Some(properties))
}

pub fn response_packet_mqtt_pubrel_success(
    protocol: &MqttProtocol,
    pkid: u16,
    reason: PubRelReason,
) -> MqttPacket {
    if !protocol.is_mqtt5() {
        return MqttPacket::PubRel(PubRel { pkid, reason: None }, None);
    }
    let rel = PubRel {
        pkid,
        reason: Some(reason),
    };
    let properties = Some(PubRelProperties::default());
    MqttPacket::PubRel(rel, properties)
}

pub fn response_packet_mqtt_pubcomp_success(protocol: &MqttProtocol, pkid: u16) -> MqttPacket {
    if !protocol.is_mqtt5() {
        return MqttPacket::PubComp(PubComp { pkid, reason: None }, None);
    }

    let rec = PubComp {
        pkid,
        reason: Some(PubCompReason::Success),
    };
    let properties = Some(PubCompProperties::default());
    MqttPacket::PubComp(rec, properties)
}

pub fn response_packet_mqtt_pubcomp_fail(
    protocol: &MqttProtocol,
    connection: &MQTTConnection,
    pkid: u16,
    reason: PubCompReason,
    reason_string: Option<String>,
) -> MqttPacket {
    if !protocol.is_mqtt5() {
        return MqttPacket::PubComp(PubComp { pkid, reason: None }, None);
    }
    let pub_ack = PubComp {
        pkid,
        reason: Some(reason),
    };
    let mut properties = PubCompProperties::default();
    if connection.is_response_problem_info() {
        properties.reason_string = reason_string;
    }
    MqttPacket::PubComp(pub_ack, Some(properties))
}

pub fn response_packet_mqtt_suback(
    protocol: &MqttProtocol,
    connection: &MQTTConnection,
    pkid: u16,
    return_codes: Vec<SubscribeReasonCode>,
    reason_string: Option<String>,
) -> MqttPacket {
    if !protocol.is_mqtt5() {
        return MqttPacket::SubAck(SubAck { pkid, return_codes }, None);
    }

    let sub_ack = SubAck { pkid, return_codes };
    let mut properties = SubAckProperties::default();
    if connection.is_response_problem_info() {
        properties.reason_string = reason_string;
    }
    MqttPacket::SubAck(sub_ack, Some(properties))
}

pub fn response_packet_mqtt_ping_resp() -> MqttPacket {
    MqttPacket::PingResp(PingResp {})
}

pub fn response_packet_mqtt_unsuback(
    connection: &MQTTConnection,
    pkid: u16,
    reasons: Vec<UnsubAckReason>,
    reason_string: Option<String>,
) -> MqttPacket {
    if reason_string.is_some() {
        debug!("{reasons:?},{reason_string:?}");
    }
    let unsub_ack = UnsubAck { pkid, reasons };
    let mut properties = UnsubAckProperties::default();
    if connection.is_response_problem_info() {
        properties.reason_string = reason_string;
    }
    MqttPacket::UnsubAck(unsub_ack, None)
}
