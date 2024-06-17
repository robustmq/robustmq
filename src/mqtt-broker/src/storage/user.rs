use metadata_struct::mqtt::user::MQTTUser;
use clients::{
    placement::mqtt::call::{placement_create_user, placement_delete_user, placement_list_user},
    poll::ClientPool,
};
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use dashmap::DashMap;
use protocol::placement_center::generate::mqtt::{
    CreateUserRequest, DeleteUserRequest, ListUserRequest,
};
use std::sync::Arc;

pub struct UserStorage {
    client_poll: Arc<ClientPool>,
}
impl UserStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return UserStorage { client_poll };
    }

    pub async fn save_user(&self, user_info: MQTTUser) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = CreateUserRequest {
            cluster_name: config.cluster_name.clone(),
            user: Some(protocol::placement_center::generate::mqtt::User {
                username: user_info.username,
                password: user_info.password,
                super_user: user_info.is_superuser,
            }),
        };
        match placement_create_user(
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

    pub async fn delete_user(&self, username: String) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = DeleteUserRequest {
            cluster_name: config.cluster_name.clone(),
            username,
        };
        match placement_delete_user(
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

    pub async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, RobustMQError> {
        let config = broker_mqtt_conf();
        let request = ListUserRequest {
            cluster_name: config.cluster_name.clone(),
            username,
        };
        match placement_list_user(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                if reply.users.len() == 0 {
                    return Ok(None);
                }
                let raw = reply.users.get(0).unwrap();
                return Ok(Some(MQTTUser {
                    username: raw.username.clone(),
                    password: raw.password.clone(),
                    is_superuser: raw.super_user,
                }));
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn user_list(&self) -> Result<DashMap<String, MQTTUser>, RobustMQError> {
        let config = broker_mqtt_conf();
        let request = ListUserRequest {
            cluster_name: config.cluster_name.clone(),
            username: "".to_string(),
        };
        match placement_list_user(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let results = DashMap::with_capacity(2);
                for raw in reply.users {
                    results.insert(
                        raw.username.clone(),
                        MQTTUser {
                            username: raw.username.clone(),
                            password: raw.password.clone(),
                            is_superuser: raw.super_user,
                        },
                    );
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
