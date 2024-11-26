#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use common_base::tools::now_second;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::session::MqttSession;

    use crate::storage::session::SessionStorage;

    #[tokio::test]
    async fn session_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let session_storage = SessionStorage::new(client_pool);
        let client_id: String = "client_id_11111".to_string();
        let session = MqttSession {
            client_id: client_id.clone(),
            session_expiry: 1000,
            broker_id: Some(1),
            reconnect_time: Some(now_second()),
            ..Default::default()
        };

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
        assert!(!result.is_empty());

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
