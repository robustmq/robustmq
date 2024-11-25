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
use common_base::error::common::CommonError;
use common_base::error::mqtt_broker::MqttBrokerError;
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

pub struct TopicStorage {
    client_pool: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        TopicStorage { client_pool }
    }

    pub async fn save_topic(&self, topic: MqttTopic) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let request = CreateTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic.topic_name.clone(),
            content: topic.encode(),
        };
        match placement_create_topic(
            self.client_pool.clone(),
            &config.placement_center,
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
            self.client_pool.clone(),
            &config.placement_center,
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
            self.client_pool.clone(),
            &config.placement_center,
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
            self.client_pool.clone(),
            &config.placement_center,
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
                    Err(e) => Err(CommonError::CommonError(e.to_string())),
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
            self.client_pool.clone(),
            &config.placement_center,
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
            self.client_pool.clone(),
            &config.placement_center,
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
                return Err(MqttBrokerError::TopicDoesNotExist(topic_name.clone()).into());
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
                    return Err(CommonError::CommonError(e.to_string()));
                }
            };
            return Ok(Some(message));
        }

        Ok(None)
    }
}
