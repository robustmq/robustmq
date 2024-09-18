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

use clients::{
    placement::mqtt::call::{placement_create_user, placement_delete_user, placement_list_user},
    poll::ClientPool,
};
use common_base::{config::broker_mqtt::broker_mqtt_conf, error::common::CommonError};
use dashmap::DashMap;
use metadata_struct::mqtt::user::MQTTUser;
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

    pub async fn save_user(&self, user_info: MQTTUser) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = CreateUserRequest {
            cluster_name: config.cluster_name.clone(),
            user_name: user_info.username.clone(),
            content: user_info.encode(),
        };
        match placement_create_user(
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

    pub async fn delete_user(&self, user_name: String) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = DeleteUserRequest {
            cluster_name: config.cluster_name.clone(),
            user_name,
        };
        match placement_delete_user(
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

    pub async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListUserRequest {
            cluster_name: config.cluster_name.clone(),
            user_name: username.clone(),
        };
        match placement_list_user(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                if reply.users.len() == 0 {
                    return Ok(None);
                }
                let raw = reply.users.get(0).unwrap();
                match serde_json::from_slice::<MQTTUser>(raw) {
                    Ok(data) => {
                        return Ok(Some(data));
                    }
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

    pub async fn user_list(&self) -> Result<DashMap<String, MQTTUser>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListUserRequest {
            cluster_name: config.cluster_name.clone(),
            user_name: "".to_string(),
        };
        match placement_list_user(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let results = DashMap::with_capacity(2);
                for raw in reply.users {
                    match serde_json::from_slice::<MQTTUser>(&raw) {
                        Ok(data) => {
                            results.insert(data.username.clone(), data);
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

#[cfg(test)]
mod tests {
    use crate::storage::user::UserStorage;
    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use std::sync::Arc;

    #[tokio::test]
    async fn user_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let user_storage = UserStorage::new(client_poll);
        let username = "test".to_string();
        let password = "test_password".to_string();
        let is_superuser = true;
        let user_info = metadata_struct::mqtt::user::MQTTUser {
            username: username.clone(),
            password: password.clone(),
            is_superuser,
        };
        user_storage.save_user(user_info).await.unwrap();

        let result = user_storage
            .get_user(username.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.username, username);
        assert_eq!(result.password, password);
        assert_eq!(result.is_superuser, is_superuser);

        let result = user_storage.user_list().await.unwrap();
        let prev_len = result.len();
        assert!(result.len() >= 1);

        user_storage.delete_user(username.clone()).await.unwrap();
        let result = user_storage.get_user(username.clone()).await.unwrap();
        assert!(result.is_none());

        let result = user_storage.get_user(username.clone()).await.unwrap();
        assert!(result.is_none());

        let result = user_storage.user_list().await.unwrap();
        assert_eq!(result.len(), prev_len - 1);
    }
}
