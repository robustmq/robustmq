use clients::{
    placement::mqtt::call::{
        placement_create_session, placement_delete_session, placement_list_session,
    },
    poll::ClientPool,
};
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use dashmap::DashMap;
use metadata_struct::mqtt::session::MQTTSession;
use protocol::placement_center::generate::mqtt::{
    CreateSessionRequest, DeleteSessionRequest, ListSessionRequest,
};
use std::sync::Arc;

pub struct SessionStorage {
    client_poll: Arc<ClientPool>,
}

impl SessionStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return SessionStorage { client_poll };
    }

    pub async fn save_session(
        &self,
        client_id: String,
        session: MQTTSession,
    ) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = CreateSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id: client_id.clone(),
            session: session.encode(),
        };
        match placement_create_session(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(format!(
                    "save user config error, error messsage:{}",
                    e.to_string()
                )))
            }
        }
    }

    pub async fn delete_session(&self, client_id: String) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = DeleteSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
        };
        match placement_delete_session(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(format!(
                    "save user config error, error messsage:{}",
                    e.to_string()
                )))
            }
        }
    }

    pub async fn get_session(
        &self,
        client_id: String,
    ) -> Result<Option<MQTTSession>, RobustMQError> {
        let config = broker_mqtt_conf();
        let request = ListSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id,
        };
        match placement_list_session(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                if reply.sessions.len() == 0 {
                    return Ok(None);
                }
                let raw = reply.sessions.get(0).unwrap();
                match serde_json::from_str::<MQTTSession>(&raw) {
                    Ok(data) => return Ok(Some(data)),
                    Err(e) => {
                        return Err(RobustMQError::CommmonError(e.to_string()));
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn list_session(&self) -> Result<DashMap<String, MQTTSession>, RobustMQError> {
        let config = broker_mqtt_conf();
        let request = ListSessionRequest {
            cluster_name: config.cluster_name.clone(),
            client_id: "".to_string(),
        };
        match placement_list_session(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let results = DashMap::with_capacity(2);
                for raw in reply.sessions {
                    match serde_json::from_str::<MQTTSession>(&raw) {
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
}
