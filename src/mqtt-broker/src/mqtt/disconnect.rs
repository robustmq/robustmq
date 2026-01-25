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
use crate::system_topic::event::{st_report_disconnected_event, StReportDisconnectedEventContext};
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    Disconnect, DisconnectProperties, DisconnectReasonCode, MqttPacket, MqttProtocol,
};
use std::sync::Arc;
use tracing::{debug, warn};

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

impl MqttService {
    pub async fn disconnect(
        &self,
        connection: &MQTTConnection,
        disconnect: &Disconnect,
        disconnect_properties: &Option<DisconnectProperties>,
    ) -> Option<MqttPacket> {
        let session =
            if let Some(session) = self.cache_manager.get_session_info(&connection.client_id) {
                st_report_disconnected_event(StReportDisconnectedEventContext {
                    storage_driver_manager: self.storage_driver_manager.clone(),
                    metadata_cache: self.cache_manager.clone(),
                    client_pool: self.client_pool.clone(),
                    session: session.clone(),
                    connection: connection.clone(),
                    connect_id: connection.connect_id,
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
