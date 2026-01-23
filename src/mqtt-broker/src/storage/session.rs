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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::mqtt::call::{
    placement_create_session, placement_delete_session, placement_get_last_will_message,
    placement_list_session, placement_save_last_will_message,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use metadata_struct::mqtt::session::MqttSession;
use protocol::meta::meta_service_mqtt::{
    CreateSessionRequest, DeleteSessionRequest, GetLastWillMessageRequest, ListSessionRequest,
    SaveLastWillMessageRequest,
};
use std::sync::Arc;

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
        let config = broker_config();
        let request = CreateSessionRequest {
            client_id,
            session: session.encode()?,
        };

        placement_create_session(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn delete_session(&self, client_id: String) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteSessionRequest { client_id };

        placement_delete_session(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn get_session(&self, client_id: String) -> Result<Option<MqttSession>, CommonError> {
        let config = broker_config();
        let request = ListSessionRequest { client_id };

        let reply =
            placement_list_session(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        if reply.sessions.is_empty() {
            return Ok(None);
        }

        let raw = reply.sessions.first().unwrap();
        let data = MqttSession::decode(raw)?;
        Ok(Some(data))
    }

    pub async fn list_session(
        &self,
        client_id: Option<String>,
    ) -> Result<DashMap<String, MqttSession>, CommonError> {
        let config = broker_config();
        let request = ListSessionRequest {
            client_id: if let Some(id) = client_id {
                id
            } else {
                "".to_string()
            },
        };

        let reply =
            placement_list_session(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        let results = DashMap::with_capacity(2);

        for raw in reply.sessions {
            let data = MqttSession::decode(&raw)?;
            results.insert(data.client_id.clone(), data);
        }

        Ok(results)
    }

    pub async fn save_last_will_message(
        &self,
        client_id: String,
        last_will_message: Vec<u8>,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = SaveLastWillMessageRequest {
            client_id,
            last_will_message,
        };

        placement_save_last_will_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;

        Ok(())
    }

    pub async fn get_last_will_message(
        &self,
        client_id: String,
    ) -> Result<Option<MqttLastWillData>, CommonError> {
        let config = broker_config();
        let request = GetLastWillMessageRequest { client_id };

        let reply = placement_get_last_will_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        if reply.message.is_empty() {
            return Ok(None);
        }

        let data = MqttLastWillData::decode(&reply.message)?;
        Ok(Some(data))
    }
}
