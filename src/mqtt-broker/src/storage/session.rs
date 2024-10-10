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

use clients::placement::mqtt::call::{
    placement_create_session, placement_delete_session, placement_list_session,
    placement_save_last_will_message, placement_update_session,
};
use clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::mqtt::session::MqttSession;
use protocol::placement_center::generate::mqtt::{
    CreateSessionRequest, DeleteSessionRequest, ListSessionRequest, SaveLastWillMessageRequest,
    UpdateSessionRequest,
};

pub struct SessionStorage {
    client_poll: Arc<ClientPool>,
}

impl SessionStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return SessionStorage { client_poll };
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
        match placement_create_session(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
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
        match placement_update_session(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn delete_session(&self, client_id: String) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = DeleteSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
        };
        match placement_delete_session(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn get_session(&self, client_id: String) -> Result<Option<MqttSession>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
        };
        match placement_list_session(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                if reply.sessions.len() == 0 {
                    return Ok(None);
                }
                let raw = reply.sessions.get(0).unwrap();
                match serde_json::from_slice::<MqttSession>(&raw) {
                    Ok(data) => return Ok(Some(data)),
                    Err(e) => {
                        return Err(CommonError::CommmonError(e.to_string()));
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn list_session(&self) -> Result<DashMap<String, MqttSession>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id: "".to_string(),
        };
        match placement_list_session(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let results = DashMap::with_capacity(2);
                for raw in reply.sessions {
                    match serde_json::from_slice::<MqttSession>(&raw) {
                        Ok(data) => {
                            results.insert(data.client_id.clone(), data);
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn save_last_will_messae(
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
        match placement_save_last_will_message(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use common_base::tools::now_second;
    use metadata_struct::mqtt::session::MqttSession;

    use crate::storage::session::SessionStorage;

    #[tokio::test]
    async fn session_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let session_storage = SessionStorage::new(client_poll);
        let client_id: String = "client_id_11111".to_string();
        let mut session = MqttSession::default();
        session.client_id = client_id.clone();
        session.session_expiry = 1000;
        session.broker_id = Some(1);
        session.reconnect_time = Some(now_second());

        session_storage
            .set_session(client_id.clone(), &session)
            .await
            .unwrap();

        let result = session_storage
            .get_session(client_id.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.client_id, client_id);
        assert_eq!(result.broker_id.unwrap(), 1);
        assert!(result.connection_id.is_none());

        session_storage
            .update_session(client_id.clone(), 3, 3, now_second(), 0)
            .await
            .unwrap();

        let result = session_storage
            .get_session(client_id.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.broker_id.unwrap(), 3);
        assert_eq!(result.connection_id.unwrap(), 3);

        let result = session_storage.list_session().await.unwrap();
        let prefix_len = result.len();
        assert!(result.len() >= 1);

        session_storage
            .delete_session(client_id.clone())
            .await
            .unwrap();

        let result = session_storage
            .get_session(client_id.clone())
            .await
            .unwrap();
        assert!(result.is_none());

        let result = session_storage.list_session().await.unwrap();
        assert_eq!(result.len(), prefix_len - 1);
    }
}
