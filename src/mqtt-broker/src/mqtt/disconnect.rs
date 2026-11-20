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

use super::MqttService;
use crate::core::cache::MQTTCacheManager;
use crate::core::connection::{
    disconnect_connection, is_request_problem_info, DisconnectConnectionContext,
};
use crate::core::event::st_report_disconnected_event;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    Disconnect, DisconnectProperties, DisconnectReasonCode, MqttPacket, MqttProtocol,
};
use std::sync::Arc;
use tracing::{debug, warn};

impl MqttService {
    pub async fn disconnect(
        &self,
        connection: &MQTTConnection,
        disconnect: &Disconnect,
        disconnect_properties: &Option<DisconnectProperties>,
    ) -> Option<MqttPacket> {
        let session = self.cache_manager.get_session_info(&connection.client_id)?;
        st_report_disconnected_event(
            &self.event_manager,
            &self.connection_manager,
            connection.connect_id,
            connection,
            &session,
            disconnect.reason_code,
        )
        .await;

        if let Err(e) = disconnect_connection(DisconnectConnectionContext {
            cache_manager: self.cache_manager.clone(),
            client_pool: self.client_pool.clone(),
            session_batcher: self.session_batcher.clone(),
            connection_manager: self.connection_manager.clone(),
            subscribe_manager: self.subscribe_manager.clone(),
            disconnect_properties: disconnect_properties.clone(),
            connection: connection.clone(),
            session: session.clone(),
            protocol: self.protocol.clone(),
        })
        .await
        {
            warn!("disconnect connection failed, {}", e.to_string());
        }

        None
    }
}

pub fn build_distinct_packet(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    protocol: &MqttProtocol,
    code: Option<DisconnectReasonCode>,
    server_reference: Option<String>,
    reason: Option<String>,
) -> MqttPacket {
    debug!(
        connect_id,
        protocol = ?protocol,
        reason_code = ?code,
        server_reference = server_reference.as_deref(),
        reason = reason.as_deref(),
        "build disconnect packet"
    );
    if !protocol.is_mqtt5() {
        return MqttPacket::Disconnect(Disconnect { reason_code: code }, None);
    }

    let mut pros = DisconnectProperties {
        server_reference,
        ..Default::default()
    };
    if is_request_problem_info(cache_manager, connect_id) {
        pros.reason_string = reason;
    }
    MqttPacket::Disconnect(Disconnect { reason_code: code }, Some(pros))
}

#[cfg(test)]
mod tests {
    use super::build_distinct_packet;
    use crate::core::tool::test_build_mqtt_cache_manager;
    use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
    use protocol::mqtt::common::{
        DisconnectProperties, DisconnectReasonCode, MqttPacket, MqttProtocol,
    };

    #[test]
    fn mqtt3_disconnect_does_not_include_mqtt5_properties() {
        let cache_manager = futures::executor::block_on(test_build_mqtt_cache_manager());
        let packet = build_distinct_packet(
            &cache_manager,
            1,
            &MqttProtocol::Mqtt3,
            Some(DisconnectReasonCode::ProtocolError),
            Some("mqtt5.example".to_string()),
            Some("details".to_string()),
        );

        assert_eq!(
            packet,
            MqttPacket::Disconnect(
                protocol::mqtt::common::Disconnect {
                    reason_code: Some(DisconnectReasonCode::ProtocolError),
                },
                None,
            )
        );
    }

    #[tokio::test]
    async fn mqtt5_disconnect_includes_server_reference_and_requested_reason() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let connection = MQTTConnection::new(ConnectionConfig {
            tenant: "tenant".to_string(),
            connect_id: 7,
            client_id: "client".to_string(),
            receive_maximum: 10,
            max_packet_size: 1024,
            topic_alias_max: 10,
            request_problem_info: 1,
            keep_alive: 60,
            source_ip_addr: "127.0.0.1:1883".to_string(),
            source_ip: "127.0.0.1".to_string(),
            clean_session: false,
        });
        cache_manager.add_connection(connection.connect_id, connection);

        let packet = build_distinct_packet(
            &cache_manager,
            7,
            &MqttProtocol::Mqtt5,
            Some(DisconnectReasonCode::ServerMoved),
            Some("mqtt5.example".to_string()),
            Some("use another broker".to_string()),
        );

        assert_eq!(
            packet,
            MqttPacket::Disconnect(
                protocol::mqtt::common::Disconnect {
                    reason_code: Some(DisconnectReasonCode::ServerMoved),
                },
                Some(DisconnectProperties {
                    server_reference: Some("mqtt5.example".to_string()),
                    reason_string: Some("use another broker".to_string()),
                    ..Default::default()
                }),
            )
        );
    }

    #[tokio::test]
    async fn mqtt5_disconnect_omits_reason_when_problem_info_is_not_requested() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let connection = MQTTConnection::new(ConnectionConfig {
            tenant: "tenant".to_string(),
            connect_id: 8,
            client_id: "client".to_string(),
            receive_maximum: 10,
            max_packet_size: 1024,
            topic_alias_max: 10,
            request_problem_info: 0,
            keep_alive: 60,
            source_ip_addr: "127.0.0.1:1883".to_string(),
            source_ip: "127.0.0.1".to_string(),
            clean_session: false,
        });
        cache_manager.add_connection(connection.connect_id, connection);

        let packet = build_distinct_packet(
            &cache_manager,
            8,
            &MqttProtocol::Mqtt5,
            None,
            None,
            Some("hidden".to_string()),
        );

        assert_eq!(
            packet,
            MqttPacket::Disconnect(
                protocol::mqtt::common::Disconnect { reason_code: None },
                Some(DisconnectProperties::default()),
            )
        );
    }
}
