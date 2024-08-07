// Copyright 2023 RobustMQ Team
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
    placement::mqtt::call::{
        placement_create_topic, placement_delete_topic, placement_list_topic,
        placement_set_topic_retain_message,
    },
    poll::ClientPool,
};
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use dashmap::DashMap;
use metadata_struct::mqtt::{message::MQTTMessage, topic::MQTTTopic};
use protocol::placement_center::generate::mqtt::{
    CreateTopicRequest, DeleteTopicRequest, ListTopicRequest, SetTopicRetainMessageRequest,
};
use std::sync::Arc;

pub struct TopicStorage {
    client_poll: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return TopicStorage { client_poll };
    }

    pub async fn save_topic(&self, topic: MQTTTopic) -> Result<(), RobustMQError> {
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
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn delete_topic(&self, topic_name: String) -> Result<(), RobustMQError> {
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
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn topic_list(&self) -> Result<DashMap<String, MQTTTopic>, RobustMQError> {
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
                    match serde_json::from_slice::<MQTTTopic>(&raw) {
                        Ok(data) => {
                            results.insert(data.topic_name.clone(), data);
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

    pub async fn get_topic(&self, topic_name: String) -> Result<Option<MQTTTopic>, RobustMQError> {
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
                if reply.topics.len() == 0 {
                    return Ok(None);
                }
                let raw = reply.topics.get(0).unwrap();
                match serde_json::from_slice::<MQTTTopic>(&raw) {
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

    pub async fn set_retain_message(
        &self,
        topic_name: &String,
        retain_message: &MQTTMessage,
        retain_message_expired_at: u64,
    ) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic_name.clone(),
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
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn delete_retain_message(&self, topic_name: &String) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic_name.clone(),
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
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    // Get the latest reserved message for the Topic dimension
    pub async fn get_retain_message(
        &self,
        topic_name: String,
    ) -> Result<Option<MQTTMessage>, RobustMQError> {
        let topic = match self.get_topic(topic_name.clone()).await {
            Ok(Some(data)) => data,
            Ok(None) => {
                return Err(RobustMQError::TopicDoesNotExist(topic_name.clone()));
            }
            Err(e) => {
                return Err(e);
            }
        };

        if let Some(retain_message) = topic.retain_message {
            if retain_message.len() == 0 {
                return Ok(None);
            }
            let message = match serde_json::from_slice::<MQTTMessage>(retain_message.as_slice()) {
                Ok(data) => data,
                Err(e) => {
                    return Err(RobustMQError::CommmonError(e.to_string()));
                }
            };
            return Ok(Some(message));
        }

        return Ok(None);
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::topic::TopicStorage;
    use bytes::Bytes;
    use clients::poll::ClientPool;
    use common_base::{
        config::broker_mqtt::init_broker_mqtt_conf_by_path, log::init_broker_mqtt_log,
        tools::unique_id,
    };
    use metadata_struct::mqtt::{message::MQTTMessage, topic::MQTTTopic};
    use protocol::mqtt::common::{Publish, PublishProperties};
    use std::sync::Arc;

    #[tokio::test]
    async fn topic_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        init_broker_mqtt_conf_by_path(&path);
        init_broker_mqtt_log();

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_poll);
        let topic_name: String = "test_password".to_string();
        let topic = MQTTTopic::new(unique_id(), topic_name.clone());
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
    }

    #[tokio::test]
    async fn topic_retain_message_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        init_broker_mqtt_conf_by_path(&path);
        init_broker_mqtt_log();

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_poll);

        let topic_name: String = "test_password".to_string();
        let client_id = unique_id();
        let content = "Robust Data".to_string();

        let mut publish = Publish::default();
        publish.payload = Bytes::from(content.clone());
        let topic = MQTTTopic::new(unique_id(), topic_name.clone());
        topic_storage.save_topic(topic).await.unwrap();

        let result_message = topic_storage
            .get_retain_message(topic_name.clone())
            .await
            .unwrap();
        assert!(result_message.is_none());

        let publish_properties = PublishProperties::default();
        let retain_message =
            MQTTMessage::build_message(&client_id, &publish, &Some(publish_properties));
        topic_storage
            .set_retain_message(&topic_name, &retain_message, 3600)
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
            .delete_retain_message(&topic_name)
            .await
            .unwrap();

        let result_message = topic_storage
            .get_retain_message(topic_name.clone())
            .await
            .unwrap();
        assert!(result_message.is_none());
    }
}
