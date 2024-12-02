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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use grpc_clients::pool::ClientPool;
    use mqtt_broker::storage::user::UserStorage;

    #[tokio::test]
    async fn user_test() {
        let path = format!("{}/../config/mqtt-server.toml", env!("CARGO_MANIFEST_DIR"));
        init_broker_mqtt_conf_by_path(&path);

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let user_storage = UserStorage::new(client_pool);
        let username = "test".to_string();
        let password = "test_password".to_string();
        let is_superuser = true;
        let user_info = metadata_struct::mqtt::user::MqttUser {
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
        assert!(!result.is_empty());

        user_storage.delete_user(username.clone()).await.unwrap();
        let result = user_storage.get_user(username.clone()).await.unwrap();
        assert!(result.is_none());

        let result = user_storage.get_user(username.clone()).await.unwrap();
        assert!(result.is_none());

        let result = user_storage.user_list().await.unwrap();
        assert_eq!(result.len(), prev_len - 1);
    }
}
