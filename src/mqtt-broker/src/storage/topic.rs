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

use common_base::config::broker_mqtt::broker_mqtt_conf;
use dashmap::DashMap;
use grpc_clients::placement::mqtt::call::{
    placement_create_topic, placement_delete_topic, placement_list_topic,
    placement_set_topic_retain_message,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::mqtt::topic::MqttTopic;
use protocol::placement_center::placement_center_mqtt::{
    CreateTopicRequest, DeleteTopicRequest, ListTopicRequest, SetTopicRetainMessageRequest,
};

use crate::handler::error::MqttBrokerError;

pub struct TopicStorage {
    client_pool: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        TopicStorage { client_pool }
    }

    pub async fn save_topic(&self, topic: MqttTopic) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = CreateTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic.topic_name.clone(),
            content: topic.encode(),
        };
        placement_create_topic(self.client_pool.clone(), &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn delete_topic(&self, topic_name: String) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = DeleteTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
        };
        placement_delete_topic(self.client_pool.clone(), &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn all(&self) -> Result<DashMap<String, MqttTopic>, MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: "".to_string(),
        };
        let reply =
            placement_list_topic(self.client_pool.clone(), &config.placement_center, request)
                .await?;
        let results = DashMap::with_capacity(2);
        for raw in reply.topics {
            let data = serde_json::from_slice::<MqttTopic>(&raw)?;
            results.insert(data.topic_name.clone(), data);
        }
        Ok(results)
    }

    pub async fn get_topic(&self, topic_name: &str) -> Result<Option<MqttTopic>, MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic_name.to_owned(),
        };

        let reply =
            placement_list_topic(self.client_pool.clone(), &config.placement_center, request)
                .await?;

        if let Some(raw) = reply.topics.first() {
            return Ok(Some(serde_json::from_slice::<MqttTopic>(raw)?));
        }

        Ok(None)
    }

    pub async fn set_retain_message(
        &self,
        topic_name: String,
        retain_message: &MqttMessage,
        retain_message_expired_at: u64,
    ) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
            retain_message: retain_message.encode(),
            retain_message_expired_at,
        };
        placement_set_topic_retain_message(
            self.client_pool.clone(),
            &config.placement_center,
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_retain_message(&self, topic_name: String) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
            retain_message: Vec::new(),
            retain_message_expired_at: 0,
        };
        placement_set_topic_retain_message(
            self.client_pool.clone(),
            &config.placement_center,
            request,
        )
        .await?;
        Ok(())
    }

    // Get the latest reserved message for the Topic dimension
    pub async fn get_retain_message(
        &self,
        topic_name: &str,
    ) -> Result<Option<MqttMessage>, MqttBrokerError> {
        if let Some(topic) = self.get_topic(topic_name).await? {
            if let Some(retain_message) = topic.retain_message {
                if retain_message.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(serde_json::from_slice::<MqttMessage>(
                    retain_message.as_slice(),
                )?));
            }
            return Ok(None);
        }
        Err(MqttBrokerError::TopicDoesNotExist(topic_name.to_owned()))
    }
}
