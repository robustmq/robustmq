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

// NOTE: kept for backward compat; use disconnect.rs.
use super::MqttService;
use crate::core::connection::{disconnect_connection, DisconnectConnectionContext};
use crate::system_topic::event::{st_report_disconnected_event, StReportDisconnectedEventContext};
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    Disconnect, DisconnectProperties, DisconnectReasonCode, MqttPacket, MqttProtocol,
};
use tracing::warn;

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

impl MqttService {
    pub async fn disconnect(
        &self,
        connect_id: u64,
        disconnect: &Disconnect,
        disconnect_properties: &Option<DisconnectProperties>,
    ) -> Option<MqttPacket> {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return None;
        };

        let session =
            if let Some(session) = self.cache_manager.get_session_info(&connection.client_id) {
                st_report_disconnected_event(StReportDisconnectedEventContext {
                    storage_driver_manager: self.storage_driver_manager.clone(),
                    metadata_cache: self.cache_manager.clone(),
                    client_pool: self.client_pool.clone(),
                    session: session.clone(),
                    connection: connection.clone(),
                    connect_id,
                    connection_manager: self.connection_manager.clone(),
                    reason: disconnect.reason_code,
                })
                .await;
                session
            } else {
                return None;
            };

        if let Err(e) = disconnect_connection(DisconnectConnectionContext {
            cache_manager: self.cache_manager.clone(),
            client_pool: self.client_pool.clone(),
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
