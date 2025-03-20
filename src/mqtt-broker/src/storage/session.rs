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

use std::sync::Arc;

use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use grpc_clients::placement::mqtt::call::{
    placement_create_session, placement_delete_session, placement_list_session,
    placement_save_last_will_message, placement_update_session,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use protocol::placement_center::placement_center_mqtt::{
    CreateSessionRequest, DeleteSessionRequest, ListSessionRequest, SaveLastWillMessageRequest,
    UpdateSessionRequest,
};

pub struct SessionStorage {
    client_pool: Arc<ClientPool>,
}

impl SessionStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        SessionStorage { client_pool }
    }

    pub async fn set_session(
        &self,
        client_id: String,
        session: &MqttSession,
    ) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = CreateSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
            session: session.encode(),
        };

        match placement_create_session(&self.client_pool, &config.placement_center, request).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn update_session(
        &self,
        client_id: String,
        connection_id: u64,
        broker_id: u64,
        reconnect_time: u64,
        distinct_time: u64,
    ) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = UpdateSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
            connection_id,
            broker_id,
            reconnect_time,
            distinct_time,
        };
        match placement_update_session(&self.client_pool, &config.placement_center, request).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_session(&self, client_id: String) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = DeleteSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
        };
        match placement_delete_session(&self.client_pool, &config.placement_center, request).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn get_session(&self, client_id: String) -> Result<Option<MqttSession>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
        };
        match placement_list_session(&self.client_pool, &config.placement_center, request).await {
            Ok(reply) => {
                if reply.sessions.is_empty() {
                    return Ok(None);
                }
                let raw = reply.sessions.first().unwrap();
                match serde_json::from_str::<MqttSession>(raw) {
                    Ok(data) => Ok(Some(data)),
                    Err(e) => Err(CommonError::CommonError(e.to_string())),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn list_session(&self) -> Result<DashMap<String, MqttSession>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id: "".to_string(),
        };
        match placement_list_session(&self.client_pool, &config.placement_center, request).await {
            Ok(reply) => {
                let results = DashMap::with_capacity(2);
                for raw in reply.sessions {
                    match serde_json::from_str::<MqttSession>(&raw) {
                        Ok(data) => {
                            results.insert(data.client_id.clone(), data);
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn save_last_will_message(
        &self,
        client_id: String,
        last_will_message: Vec<u8>,
    ) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = SaveLastWillMessageRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
            last_will_message,
        };
        match placement_save_last_will_message(&self.client_pool, &config.placement_center, request)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
