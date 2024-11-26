#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use grpc_clients::pool::ClientPool;

    use crate::storage::user::UserStorage;

    #[tokio::test]
    async fn user_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
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
