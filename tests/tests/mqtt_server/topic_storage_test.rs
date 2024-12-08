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

    use bytes::Bytes;
    use common_base::config::broker_mqtt::{broker_mqtt_conf, init_broker_mqtt_conf_by_path};
    use common_base::logs::init_log;
    use common_base::tools::unique_id;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::message::MqttMessage;
    use metadata_struct::mqtt::topic::MqttTopic;
    use mqtt_broker::storage::topic::TopicStorage;
    use protocol::mqtt::common::{Publish, PublishProperties};

    #[tokio::test]
    async fn topic_test() {
        let path = format!("{}/../config/mqtt-server.toml", env!("CARGO_MANIFEST_DIR"));
        let log_config = format!("{}/../config/log4rs.yaml", env!("CARGO_MANIFEST_DIR"));
        let log_path = format!("{}/../logs/tests", env!("CARGO_MANIFEST_DIR"));

        init_broker_mqtt_conf_by_path(&path);
        init_log(&log_config, &log_path);

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_pool);

        let mqtt_conf = broker_mqtt_conf();

        let topic_name: String = "test_password".to_string();
        let topic = MqttTopic::new(
            unique_id(),
            mqtt_conf.cluster_name.clone(),
            topic_name.clone(),
        );
        topic_storage.save_topic(topic).await.unwrap();

        let result = topic_storage.get_topic(&topic_name).await.unwrap().unwrap();
        assert!(result.retain_message.is_none());
        assert_eq!(result.topic_name, topic_name);
        assert!(!result.topic_id.is_empty());

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
        let path = format!("{}/../config/mqtt-server.toml", env!("CARGO_MANIFEST_DIR"));

        init_broker_mqtt_conf_by_path(&path);

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_pool);

        let topic_name: String = unique_id();
        let client_id = unique_id();
        let content = "Robust Data".to_string();

        let publish = Publish {
            payload: Bytes::from(content.clone()),
            ..Default::default()
        };

        let mqtt_conf = broker_mqtt_conf();

        let topic = MqttTopic::new(
            unique_id(),
            mqtt_conf.cluster_name.clone(),
            topic_name.clone(),
        );
        topic_storage.save_topic(topic).await.unwrap();

        let result = topic_storage.get_topic(&topic_name).await.unwrap().unwrap();
        println!("{:?}", result);

        let result_message = topic_storage.get_retain_message(&topic_name).await.unwrap();
        assert!(result_message.is_none());

        let publish_properties = PublishProperties::default();
        let retain_message =
            MqttMessage::build_message(&client_id, &publish, &Some(publish_properties), 600);
        topic_storage
            .set_retain_message(topic_name.clone(), &retain_message, 3600)
            .await
            .unwrap();

        let result_message = topic_storage
            .get_retain_message(&topic_name)
            .await
            .unwrap()
            .unwrap();
        let payload = String::from_utf8(result_message.payload.to_vec()).unwrap();
        assert_eq!(payload, content);

        topic_storage
            .delete_retain_message(topic_name.clone())
            .await
            .unwrap();

        let result_message = topic_storage.get_retain_message(&topic_name).await.unwrap();
        assert!(result_message.is_none());
    }
}
