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

use crate::core::connection::{disconnect_connection, is_delete_session};
use crate::system_topic::event::{st_report_disconnected_event, StReportDisconnectedEventContext};

use protocol::mqtt::common::{Disconnect, DisconnectProperties, MqttPacket};
use tracing::warn;

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
        }

        let delete_session = if let Some(properties) = disconnect_properties {
            is_delete_session(&properties.user_properties)
        } else {
            false
        };

        if let Err(e) = disconnect_connection(
            &connection.client_id,
            connect_id,
            &self.cache_manager,
            &self.client_pool,
            &self.connection_manager,
            &self.subscribe_manager,
            delete_session,
        )
        .await
        {
            warn!("disconnect connection failed, {}", e.to_string());
        }

        None
    }
}
