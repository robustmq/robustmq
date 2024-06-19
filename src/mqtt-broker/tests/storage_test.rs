mod common;
#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use clients::poll::ClientPool;
    use metadata_struct::mqtt::{cluster::MQTTCluster, message::MQTTMessage, session::MQTTSession};
    use mqtt_broker::storage::{
        cluster::ClusterStorage, session::SessionStorage, topic::TopicStorage, user::UserStorage,
    };
    use protocol::mqtt::{Publish, PublishProperties};
    use std::sync::Arc;
    use crate::common::setup;

    #[tokio::test]
    async fn user_test() {
        setup();
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
        let result = user_storage.user_list().await.unwrap();
        assert_eq!(result.len(), prev_len - 1);
    }

    #[tokio::test]
    async fn topic_test() {
        setup();
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_poll);
        let topic_name: String = "test_password".to_string();
        topic_storage.save_topic(topic_name.clone()).await.unwrap();

        let result = topic_storage
            .get_topic(topic_name.clone())
            .await
            .unwrap()
            .unwrap();
        assert!(result.retain_message.is_none());
        assert_eq!(result.topic_name, topic_name);
        assert!(!result.topic_id.is_empty());

        let result = topic_storage.topic_list().await.unwrap();
        let prefix_len = result.len();
        assert!(result.len() >= 1);

        topic_storage
            .delete_topic(topic_name.clone())
            .await
            .unwrap();
        let result = topic_storage.get_topic(topic_name.clone()).await.unwrap();
        assert!(result.is_none());

        let result = topic_storage.topic_list().await.unwrap();
        assert_eq!(result.len(), prefix_len - 1);

        let topic_name: String = "test_password".to_string();
        topic_storage.save_topic(topic_name.clone()).await.unwrap();

        let result_message = topic_storage
            .get_retain_message(topic_name.clone())
            .await
            .unwrap();
        assert!(result_message.is_none());

        let client_id = "cid_test_1".to_string();
        let mut publish = Publish::default();
        publish.payload = Bytes::from("Robust Data");

        let publish_properties = PublishProperties::default();
        let retain_message =
            MQTTMessage::build_message(client_id, publish, Some(publish_properties));
        topic_storage
            .save_retain_message(topic_name.clone(), retain_message)
            .await
            .unwrap();

        let result_message = topic_storage
            .get_retain_message(topic_name.clone())
            .await
            .unwrap()
            .unwrap();
        let payload = String::from_utf8(result_message.payload.to_vec()).unwrap();
        assert_eq!(payload, "Robust Data");
    }

    #[tokio::test]
    async fn session_test() {
        setup();
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let session_storage = SessionStorage::new(client_poll);
        let client_id: String = "client_id_11111".to_string();
        let mut session = MQTTSession::default();
        session.client_id = client_id.clone();
        session_storage
            .save_session(client_id.clone(), session)
            .await
            .unwrap();

        let result = session_storage
            .get_session(client_id.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.client_id, client_id);

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

    #[tokio::test]
    async fn cluster_test() {
        setup();
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let cluster_storage = ClusterStorage::new(client_poll);

        let cluster_name = "robust_test".to_string();
        let mut cluster = MQTTCluster::default();
        cluster.topic_alias_max = 999;
        cluster_storage
            .set_cluster_config(cluster_name.clone(), cluster)
            .await
            .unwrap();

        let result = cluster_storage
            .get_cluster_config(cluster_name.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.topic_alias_max(), 999);

        cluster_storage
            .delete_cluster_config(cluster_name.clone())
            .await
            .unwrap();

        let result = cluster_storage
            .get_cluster_config(cluster_name.clone())
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
