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

use super::{MqttService, MqttServiceConnectContext};
use crate::core::cache::ConnectionLiveTime;
use crate::core::connection::response_information;
use crate::core::connection::{build_connection, get_client_id};
use crate::core::content_type::payload_format_indicator_check_by_lastwill;
use crate::core::error::MqttBrokerError;
use crate::core::flapping_detect::check_flapping_detect;
use crate::core::last_will::save_last_will_message;
use crate::core::session::{session_process, BuildSessionContext};
use crate::core::string_validator::{validate_client_id, validate_password, validate_username};
use crate::core::sub_auto::try_auto_subscribe;
use crate::core::topic::topic_name_validator;
use crate::system_topic::event::{st_report_connected_event, StReportConnectedEventContext};
use common_base::tools::now_second;
use common_config::config::BrokerConfig;
use common_metrics::mqtt::auth::{record_mqtt_auth_failed, record_mqtt_auth_success};
use protocol::mqtt::common::{
    ConnAck, ConnAckProperties, Connect, ConnectProperties, ConnectReturnCode, LastWill,
    LastWillProperties, Login, MqttPacket, MqttProtocol,
};
use std::cmp::min;
use tracing::debug;

impl MqttService {
    pub async fn connect(&self, context: MqttServiceConnectContext) -> MqttPacket {
        let cluster = self.cache_manager.broker_cache.get_cluster_config().await;

        if let Some(res) = connect_validator(
            &self.protocol,
            &cluster,
            &context.connect,
            &context.connect_properties,
            &context.last_will,
            &context.last_will_properties,
            &context.login,
        ) {
            return res;
        }

        // client id
        let (data, resp) = get_client_id(
            &self.protocol,
            context.connect.clean_session,
            &context.connect.client_id,
            &context.connect_properties,
        );

        if let Some(pkt) = resp {
            return pkt;
        }

        let Some((client_id, new_client_id)) = data else {
            return build_connect_ack_fail_packet(
                &self.protocol,
                ConnectReturnCode::UnspecifiedError,
                &context.connect_properties,
                Some("get_client_id returned empty result".to_string()),
            );
        };

        let connection = build_connection(
            context.connect_id,
            client_id.clone(),
            &self.cache_manager,
            &context.connect,
            &context.connect_properties,
            &context.addr,
        )
        .await;

        // flapping detect check
        if cluster.mqtt_flapping_detect.enable {
            if let Err(e) = check_flapping_detect(
                context.connect.client_id.clone(),
                &self.cache_manager,
                &self.rocksdb_engine_handler,
            )
            .await
            {
                return build_connect_ack_fail_packet(
                    &self.protocol,
                    ConnectReturnCode::UnspecifiedError,
                    &context.connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        // auth check
        if self.auth_driver.auth_connect_check(&connection).await {
            return build_connect_ack_fail_packet(
                &self.protocol,
                ConnectReturnCode::Banned,
                &context.connect_properties,
                Some("client is banned".to_string()),
            );
        }

        // login check
        match self
            .auth_driver
            .auth_login_check(
                &context.login,
                &context.connect_properties,
                &context.addr,
                Some(&context.connect.client_id),
            )
            .await
        {
            Ok(flag) => {
                if !flag {
                    record_mqtt_auth_failed();
                    return build_connect_ack_fail_packet(
                        &self.protocol,
                        ConnectReturnCode::NotAuthorized,
                        &context.connect_properties,
                        Some("login not authorized".to_string()),
                    );
                }
                record_mqtt_auth_success();
            }
            Err(e) => {
                return build_connect_ack_fail_packet(
                    &self.protocol,
                    ConnectReturnCode::UnspecifiedError,
                    &context.connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        // session process
        let (session, new_session) = match session_process(
            &self.protocol,
            BuildSessionContext {
                connect_id: context.connect_id,
                client_id: client_id.clone(),
                connect: context.connect.clone(),
                connect_properties: context.connect_properties.clone(),
                last_will: context.last_will.clone(),
                last_will_properties: context.last_will_properties.clone(),
                client_pool: self.client_pool.clone(),
                cache_manager: self.cache_manager.clone(),
                subscribe_manager: self.subscribe_manager.clone(),
            },
        )
        .await
        {
            Ok((session, new_session)) => (session, new_session),
            Err(e) => {
                return build_connect_ack_fail_packet(
                    &self.protocol,
                    ConnectReturnCode::MalformedPacket,
                    &context.connect_properties,
                    Some(e.to_string()),
                );
            }
        };

        if let Err(e) = save_last_will_message(
            client_id.clone(),
            &context.last_will,
            &context.last_will_properties,
            &self.client_pool,
        )
        .await
        {
            return build_connect_ack_fail_packet(
                &self.protocol,
                ConnectReturnCode::UnspecifiedError,
                &context.connect_properties,
                Some(e.to_string()),
            );
        }

        if let Err(e) = try_auto_subscribe(
            client_id.clone(),
            &context.login,
            &self.protocol,
            &self.client_pool,
            &self.cache_manager,
            &self.subscribe_manager,
        )
        .await
        {
            return build_connect_ack_fail_packet(
                &self.protocol,
                ConnectReturnCode::UnspecifiedError,
                &context.connect_properties,
                Some(e.to_string()),
            );
        }

        let live_time = ConnectionLiveTime {
            protocol: self.protocol.clone(),
            keep_live: context.connect.keep_alive,
            heartbeat: now_second(),
        };
        self.cache_manager
            .report_heartbeat(client_id.clone(), live_time);

        self.cache_manager.add_session(&client_id, &session);
        self.cache_manager
            .add_connection(context.connect_id, connection.clone());

        st_report_connected_event(StReportConnectedEventContext {
            storage_driver_manager: self.storage_driver_manager.clone(),
            metadata_cache: self.cache_manager.clone(),
            client_pool: self.client_pool.clone(),
            session: session.clone(),
            connection: connection.clone(),
            connect_id: context.connect_id,
            connection_manager: self.connection_manager.clone(),
        })
        .await;

        build_connect_ack_success_packet(ResponsePacketMqttConnectSuccessContext {
            protocol: self.protocol.clone(),
            cluster: cluster.clone(),
            client_id: client_id.clone(),
            auto_client_id: new_client_id,
            session_expiry_interval: session.session_expiry_interval as u32,
            session_present: !new_session,
            keep_alive: connection.keep_alive,
            connect_properties: context.connect_properties.clone(),
        })
    }
}

#[derive(Clone)]
struct ResponsePacketMqttConnectSuccessContext {
    pub protocol: MqttProtocol,
    pub cluster: BrokerConfig,
    pub client_id: String,
    pub auto_client_id: bool,
    pub session_expiry_interval: u32,
    pub session_present: bool,
    pub keep_alive: u16,
    pub connect_properties: Option<ConnectProperties>,
}

fn build_connect_ack_success_packet(
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
        max_qos: Some(context.cluster.mqtt_protocol_config.max_qos_flight_message),
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

pub fn build_connect_ack_fail_packet(
    protocol: &MqttProtocol,
    code: ConnectReturnCode,
    connect_properties: &Option<ConnectProperties>,
    error_reason: Option<String>,
) -> MqttPacket {
    debug!(
        protocol = ?protocol,
        reason_code = ?code,
        reason = error_reason.as_deref(),
        "build connect ack fail packet"
    );

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
    let is_request_problem_info = if let Some(pros) = connect_properties {
        if let Some(problem) = pros.request_problem_info {
            problem == 1
        } else {
            false
        }
    } else {
        false
    };

    if is_request_problem_info {
        properties.reason_string = error_reason;
    }

    MqttPacket::ConnAck(
        ConnAck {
            session_present: false,
            code,
        },
        Some(properties),
    )
}

fn connect_validator(
    protocol: &MqttProtocol,
    cluster: &BrokerConfig,
    connect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    login: &Option<Login>,
) -> Option<MqttPacket> {
    if cluster.mqtt_security.is_self_protection_status {
        return Some(build_connect_ack_fail_packet(
            protocol,
            ConnectReturnCode::ServerBusy,
            connect_properties,
            Some(MqttBrokerError::ClusterIsInSelfProtection.to_string()),
        ));
    }

    if !connect.client_id.is_empty() {
        if let Err(e) = validate_client_id(&connect.client_id, protocol.is_mqtt5()) {
            return Some(build_connect_ack_fail_packet(
                protocol,
                ConnectReturnCode::ClientIdentifierNotValid,
                connect_properties,
                Some(e),
            ));
        }
    }

    if let Some(login_info) = login {
        if let Err(e) = validate_username(&login_info.username) {
            return Some(build_connect_ack_fail_packet(
                protocol,
                ConnectReturnCode::BadUserNamePassword,
                connect_properties,
                Some(e),
            ));
        }
        if let Err(e) = validate_password(&login_info.password) {
            return Some(build_connect_ack_fail_packet(
                protocol,
                ConnectReturnCode::BadUserNamePassword,
                connect_properties,
                Some(e),
            ));
        }
    }

    if let Some(properties) = connect_properties {
        if let Some(receive_max) = properties.receive_maximum {
            if receive_max == 0 {
                return Some(build_connect_ack_fail_packet(
                    protocol,
                    ConnectReturnCode::ProtocolError,
                    connect_properties,
                    Some("receive_maximum must not be 0".to_string()),
                ));
            }
        }

        if let Some(max_packet_size) = properties.max_packet_size {
            if max_packet_size == 0 {
                return Some(build_connect_ack_fail_packet(
                    protocol,
                    ConnectReturnCode::ProtocolError,
                    connect_properties,
                    Some("max_packet_size must not be 0".to_string()),
                ));
            }
        }

        if let Some(request_response_info) = properties.request_response_info {
            if request_response_info > 1 {
                return Some(build_connect_ack_fail_packet(
                    protocol,
                    ConnectReturnCode::ProtocolError,
                    connect_properties,
                    Some("request_response_info must be 0 or 1".to_string()),
                ));
            }
        }

        if let Some(request_problem_info) = properties.request_problem_info {
            if request_problem_info > 1 {
                return Some(build_connect_ack_fail_packet(
                    protocol,
                    ConnectReturnCode::ProtocolError,
                    connect_properties,
                    Some("request_problem_info must be 0 or 1".to_string()),
                ));
            }
        }
    }

    if let Some(will) = last_will {
        if will.topic.is_empty() {
            return Some(build_connect_ack_fail_packet(
                protocol,
                ConnectReturnCode::TopicNameInvalid,
                connect_properties,
                Some("will topic is empty".to_string()),
            ));
        }

        let topic_name = match String::from_utf8(will.topic.to_vec()) {
            Ok(da) => da,
            Err(e) => {
                return Some(build_connect_ack_fail_packet(
                    protocol,
                    ConnectReturnCode::TopicNameInvalid,
                    connect_properties,
                    Some(e.to_string()),
                ));
            }
        };

        if let Err(e) = topic_name_validator(&topic_name) {
            return Some(build_connect_ack_fail_packet(
                protocol,
                ConnectReturnCode::TopicNameInvalid,
                connect_properties,
                Some(e.to_string()),
            ));
        }

        if !payload_format_indicator_check_by_lastwill(last_will, last_will_properties) {
            return Some(build_connect_ack_fail_packet(
                protocol,
                ConnectReturnCode::PayloadFormatInvalid,
                connect_properties,
                Some("will payload format invalid".to_string()),
            ));
        }

        let max_packet_size = connection_max_packet_size(connect_properties, cluster) as usize;
        if will.message.len() > max_packet_size {
            return Some(build_connect_ack_fail_packet(
                protocol,
                ConnectReturnCode::PacketTooLarge,
                connect_properties,
                Some("will payload exceeds max packet size".to_string()),
            ));
        }
    }
    None
}

fn connection_max_packet_size(
    connect_properties: &Option<ConnectProperties>,
    cluster: &BrokerConfig,
) -> u32 {
    if let Some(properties) = connect_properties {
        if let Some(size) = properties.max_packet_size {
            return min(size, cluster.mqtt_protocol_config.max_packet_size);
        }
    }
    cluster.mqtt_protocol_config.max_packet_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use protocol::mqtt::common::{QoS, MqttProtocol};

    fn build_test_connect(client_id: &str) -> Connect {
        Connect {
            keep_alive: 60,
            client_id: client_id.to_string(),
            clean_session: true,
        }
    }

    fn build_test_properties(
        receive_maximum: Option<u16>,
        max_packet_size: Option<u32>,
        request_response_info: Option<u8>,
        request_problem_info: Option<u8>,
    ) -> ConnectProperties {
        ConnectProperties {
            receive_maximum,
            max_packet_size,
            request_response_info,
            request_problem_info,
            ..Default::default()
        }
    }

    fn build_test_last_will(topic: &str, message: &[u8]) -> LastWill {
        LastWill {
            topic: Bytes::from(topic.to_string()),
            message: Bytes::from(message.to_vec()),
            qos: QoS::AtMostOnce,
            retain: false,
        }
    }

    fn build_test_login(username: &str, password: &str) -> Login {
        Login {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    #[test]
    fn test_client_id_mqtt3_too_long() {
        let protocol = MqttProtocol::Mqtt3;
        let cluster = common_config::broker::default_broker_config();
        let long_id = "a".repeat(24);
        let connect = build_test_connect(&long_id);

        let result = connect_validator(&protocol, &cluster, &connect, &None, &None, &None, &None);
        assert!(result.is_some());
    }

    #[test]
    fn test_client_id_mqtt5_long_valid() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let long_id = "a".repeat(100);
        let connect = build_test_connect(&long_id);

        let result = connect_validator(&protocol, &cluster, &connect, &None, &None, &None, &None);
        assert!(result.is_none());
    }

    #[test]
    fn test_client_id_with_null_char() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("client\0id");

        let result = connect_validator(&protocol, &cluster, &connect, &None, &None, &None, &None);
        assert!(result.is_some());
    }

    #[test]
    fn test_username_empty() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let login = Some(build_test_login("", "password"));

        let result = connect_validator(&protocol, &cluster, &connect, &None, &None, &None, &login);
        assert!(result.is_some());
    }

    #[test]
    fn test_password_empty() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let login = Some(build_test_login("username", ""));

        let result = connect_validator(&protocol, &cluster, &connect, &None, &None, &None, &login);
        assert!(result.is_some());
    }

    #[test]
    fn test_username_with_null_char() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let login = Some(build_test_login("user\0name", "password"));

        let result = connect_validator(&protocol, &cluster, &connect, &None, &None, &None, &login);
        assert!(result.is_some());
    }

    #[test]
    fn test_receive_maximum_zero() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let properties = Some(build_test_properties(Some(0), None, None, None));

        let result = connect_validator(&protocol, &cluster, &connect, &properties, &None, &None, &None);
        assert!(result.is_some());
    }

    #[test]
    fn test_max_packet_size_zero() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let properties = Some(build_test_properties(None, Some(0), None, None));

        let result = connect_validator(&protocol, &cluster, &connect, &properties, &None, &None, &None);
        assert!(result.is_some());
    }

    #[test]
    fn test_request_response_info_invalid() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let properties = Some(build_test_properties(None, None, Some(2), None));

        let result = connect_validator(&protocol, &cluster, &connect, &properties, &None, &None, &None);
        assert!(result.is_some());
    }

    #[test]
    fn test_request_problem_info_invalid() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let properties = Some(build_test_properties(None, None, None, Some(2)));

        let result = connect_validator(&protocol, &cluster, &connect, &properties, &None, &None, &None);
        assert!(result.is_some());
    }

    #[test]
    fn test_will_topic_empty() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let will = Some(build_test_last_will("", b"test message"));

        let result = connect_validator(&protocol, &cluster, &connect, &None, &will, &None, &None);
        assert!(result.is_some());
    }

    #[test]
    fn test_will_message_empty_is_valid() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let will = Some(build_test_last_will("test/topic", b""));

        let result = connect_validator(&protocol, &cluster, &connect, &None, &will, &None, &None);
        assert!(result.is_none(), "Empty will message should be valid");
    }

    #[test]
    fn test_valid_connect() {
        let protocol = MqttProtocol::Mqtt5;
        let cluster = common_config::broker::default_broker_config();
        let connect = build_test_connect("test_client");
        let properties = Some(build_test_properties(Some(100), Some(1024), Some(1), Some(1)));

        let result = connect_validator(&protocol, &cluster, &connect, &properties, &None, &None, &None);
        assert!(result.is_none());
    }
}
