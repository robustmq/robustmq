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

 use grpc_clients::placement::mqtt::call::{
    placement_create_topic, placement_delete_topic, placement_list_topic,
    placement_set_topic_retain_message,
};
 use grpc_clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use common_base::error::mqtt_broker::MQTTBrokerError;
use dashmap::DashMap;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::mqtt::topic::MqttTopic;
use protocol::placement_center::generate::mqtt::{
    CreateTopicRequest, DeleteTopicRequest, ListTopicRequest, SetTopicRetainMessageRequest,
};

pub struct TopicStorage {
    client_poll: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        TopicStorage { client_poll }
    }

    pub async fn save_topic(&self, topic: MqttTopic) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = CreateTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic.topic_name.clone(),
            content: topic.encode(),
        };
        match placement_create_topic(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_topic(&self, topic_name: String) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = DeleteTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
        };
        match placement_delete_topic(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn topic_list(&self) -> Result<DashMap<String, MqttTopic>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: "".to_string(),
        };
        match placement_list_topic(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let results = DashMap::with_capacity(2);
                for raw in reply.topics {
                    match serde_json::from_slice::<MqttTopic>(&raw) {
                        Ok(data) => {
                            results.insert(data.topic_name.clone(), data);
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_topic(&self, topic_name: String) -> Result<Option<MqttTopic>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
        };
        match placement_list_topic(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                if reply.topics.is_empty() {
                    return Ok(None);
                }
                let raw = reply.topics.first().unwrap();
                match serde_json::from_slice::<MqttTopic>(raw) {
                    Ok(data) => Ok(Some(data)),
                    Err(e) => Err(CommonError::CommmonError(e.to_string())),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn set_retain_message(
        &self,
        topic_name: String,
        retain_message: &MqttMessage,
        retain_message_expired_at: u64,
    ) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
            retain_message: retain_message.encode(),
            retain_message_expired_at,
        };
        match placement_set_topic_retain_message(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_retain_message(&self, topic_name: String) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
            retain_message: Vec::new(),
            retain_message_expired_at: 0,
        };
        match placement_set_topic_retain_message(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // Get the latest reserved message for the Topic dimension
    pub async fn get_retain_message(
        &self,
        topic_name: String,
    ) -> Result<Option<MqttMessage>, CommonError> {
        let topic = match self.get_topic(topic_name.clone()).await {
            Ok(Some(data)) => data,
            Ok(None) => {
                return Err(MQTTBrokerError::TopicDoesNotExist(topic_name.clone()).into());
            }
            Err(e) => {
                return Err(e);
            }
        };

        if let Some(retain_message) = topic.retain_message {
            if retain_message.is_empty() {
                return Ok(None);
            }
            let message = match serde_json::from_slice::<MqttMessage>(retain_message.as_slice()) {
                Ok(data) => data,
                Err(e) => {
                    return Err(CommonError::CommmonError(e.to_string()));
                }
            };
            return Ok(Some(message));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
     use grpc_clients::poll::ClientPool;
    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use common_base::logs::init_log;
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::message::MqttMessage;
    use metadata_struct::mqtt::topic::MqttTopic;
    use protocol::mqtt::common::{Publish, PublishProperties};

    use crate::storage::topic::TopicStorage;

    #[tokio::test]
    async fn topic_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        let log_config = format!("{}/../../config/log4rs.yaml", env!("CARGO_MANIFEST_DIR"));
        let log_path = format!("{}/../../logs/tests", env!("CARGO_MANIFEST_DIR"));

        init_broker_mqtt_conf_by_path(&path);
        init_log(&log_config, &log_path);

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_poll);
        let topic_name: String = "test_password".to_string();
        let topic = MqttTopic::new(unique_id(), topic_name.clone());
        topic_storage.save_topic(topic).await.unwrap();

        let result = topic_storage
            .get_topic(topic_name.clone())
            .await
            .unwrap()
            .unwrap();
        assert!(result.retain_message.is_none());
        assert_eq!(result.topic_name, topic_name);
        assert!(!result.topic_id.is_empty());

        let result = topic_storage.topic_list().await.unwrap();
        assert!(!result.is_empty());

        topic_storage
            .delete_topic(topic_name.clone())
            .await
            .unwrap();

        let result = topic_storage.get_topic(topic_name.clone()).await.unwrap();
        assert!(result.is_none());

        topic_storage.topic_list().await.unwrap();
    }

    #[tokio::test]
    async fn topic_retain_message_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        init_broker_mqtt_conf_by_path(&path);

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_poll);

        let topic_name: String = unique_id();
        let client_id = unique_id();
        let content = "Robust Data".to_string();

        let publish = Publish {
            payload: Bytes::from(content.clone()),
            ..Default::default()
        };
        let topic = MqttTopic::new(unique_id(), topic_name.clone());
        topic_storage.save_topic(topic).await.unwrap();

        let result_message = topic_storage
            .get_retain_message(topic_name.clone())
            .await
            .unwrap();
        assert!(result_message.is_none());

        let publish_properties = PublishProperties::default();
        let retain_message =
            MqttMessage::build_message(&client_id, &publish, &Some(publish_properties), 600);
        topic_storage
            .set_retain_message(topic_name.clone(), &retain_message, 3600)
            .await
            .unwrap();

        let result_message = topic_storage
            .get_retain_message(topic_name.clone())
            .await
            .unwrap()
            .unwrap();
        let payload = String::from_utf8(result_message.payload.to_vec()).unwrap();
        assert_eq!(payload, content);

        topic_storage
            .delete_retain_message(topic_name.clone())
            .await
            .unwrap();

        let result_message = topic_storage
            .get_retain_message(topic_name.clone())
            .await
            .unwrap();
        assert!(result_message.is_none());
    }
}
