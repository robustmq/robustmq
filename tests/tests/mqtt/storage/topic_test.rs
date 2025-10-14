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
    use bytes::Bytes;
    use common_base::tools::unique_id;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::message::MqttMessage;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use mqtt_broker::storage::topic::TopicStorage;
    use protocol::mqtt::common::{Publish, PublishProperties};
    use std::sync::Arc;

    #[tokio::test]
    async fn topic_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_pool);

        let topic_name: String = "test_password".to_string();
        let topic = MQTTTopic::new(config.cluster_name.clone(), topic_name.clone());
        match topic_storage.save_topic(topic).await {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        }

        let result = topic_storage.get_topic(&topic_name).await.unwrap().unwrap();
        assert_eq!(result.topic_name, topic_name);
        assert!(!result.topic_name.is_empty());

        let result = topic_storage.all().await.unwrap();
        assert!(!result.is_empty());

        topic_storage
            .delete_topic(topic_name.clone())
            .await
            .unwrap();

        let result = topic_storage.get_topic(&topic_name).await.unwrap();
        assert!(result.is_none());

        topic_storage.all().await.unwrap();
    }

    #[tokio::test]
    async fn topic_retain_message_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_pool);

        let topic_name: String = unique_id();
        let client_id = unique_id();
        let content = "Robust Data".to_string();
        let publish = Publish {
            payload: Bytes::from(content.clone()),
            ..Default::default()
        };

        let topic = MQTTTopic::new(config.cluster_name.clone(), topic_name.clone());
        println!("{:?}", topic);
        topic_storage.save_topic(topic.clone()).await.unwrap();

        let publish_properties = PublishProperties::default();
        let retain_message =
            MqttMessage::build_message(&client_id, &publish, &Some(publish_properties), 600);

        topic_storage
            .set_retain_message(topic.topic_name.clone(), &retain_message, 1000)
            .await
            .unwrap();

        let (result_message, result_message_qt) = topic_storage
            .get_retain_message(&topic.topic_name)
            .await
            .unwrap();
        println!("{:?}", result_message);
        println!("{:?}", result_message_qt);
        assert!(result_message.is_some());
        assert!(result_message_qt.is_some());

        let msg = serde_json::from_str::<MqttMessage>(&result_message.unwrap()).unwrap();
        let payload = String::from_utf8(msg.payload.to_vec()).unwrap();
        assert_eq!(payload, content);

        topic_storage
            .delete_retain_message(topic_name.clone())
            .await
            .unwrap();

        let (result_message, result_message_at) =
            topic_storage.get_retain_message(&topic_name).await.unwrap();
        assert!(result_message.is_none());
        assert!(result_message_at.is_none());
    }
}
