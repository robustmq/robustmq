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

use common_config::mqtt::broker_mqtt_conf;
use dashmap::DashMap;
use grpc_clients::placement::mqtt::call::{
    placement_create_topic, placement_create_topic_rewrite_rule, placement_delete_topic,
    placement_delete_topic_rewrite_rule, placement_list_topic, placement_list_topic_rewrite_rule,
    placement_set_topic_retain_message,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::placement_center::placement_center_mqtt::{
    CreateTopicRequest, CreateTopicRewriteRuleRequest, DeleteTopicRequest,
    DeleteTopicRewriteRuleRequest, ListTopicRequest, ListTopicRewriteRuleRequest,
    SetTopicRetainMessageRequest,
};
use std::sync::Arc;

use crate::handler::error::MqttBrokerError;

pub struct TopicStorage {
    client_pool: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        TopicStorage { client_pool }
    }

    pub async fn save_topic(&self, topic: MQTTTopic) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = CreateTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic.topic_name.clone(),
            content: topic.encode(),
        };
        placement_create_topic(&self.client_pool, &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn delete_topic(&self, topic_name: String) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = DeleteTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
        };
        placement_delete_topic(&self.client_pool, &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn all(&self) -> Result<DashMap<String, MQTTTopic>, MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: "".to_string(),
        };
        let mut data_stream =
            placement_list_topic(&self.client_pool, &config.placement_center, request).await?;
        let results = DashMap::with_capacity(2);

        while let Some(data) = data_stream.message().await? {
            let topic = serde_json::from_slice::<MQTTTopic>(&data.topic)?;
            results.insert(topic.topic_name.clone(), topic);
        }

        Ok(results)
    }

    pub async fn get_topic(&self, topic_name: &str) -> Result<Option<MQTTTopic>, MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic_name.to_owned(),
        };

        let mut data_stream =
            placement_list_topic(&self.client_pool, &config.placement_center, request).await?;

        if let Some(data) = data_stream.message().await? {
            let topic = serde_json::from_slice::<MQTTTopic>(data.topic.as_slice())?;
            return Ok(Some(topic));
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
        placement_set_topic_retain_message(&self.client_pool, &config.placement_center, request)
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
        placement_set_topic_retain_message(&self.client_pool, &config.placement_center, request)
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

    pub async fn all_topic_rewrite_rule(
        &self,
    ) -> Result<Vec<MqttTopicRewriteRule>, MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRewriteRuleRequest {
            cluster_name: config.cluster_name.clone(),
        };
        let reply =
            placement_list_topic_rewrite_rule(&self.client_pool, &config.placement_center, request)
                .await?;
        let mut results = Vec::with_capacity(8);
        for raw in reply.topic_rewrite_rules {
            let data = serde_json::from_slice::<MqttTopicRewriteRule>(&raw)?;
            results.push(data);
        }
        Ok(results)
    }

    pub async fn create_topic_rewrite_rule(
        &self,
        req: MqttTopicRewriteRule,
    ) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = CreateTopicRewriteRuleRequest {
            cluster_name: config.cluster_name.clone(),
            action: req.action.clone(),
            source_topic: req.source_topic.clone(),
            dest_topic: req.dest_topic.clone(),
            regex: req.regex.clone(),
        };
        placement_create_topic_rewrite_rule(&self.client_pool, &config.placement_center, request)
            .await?;
        Ok(())
    }

    pub async fn delete_topic_rewrite_rule(
        &self,
        action: String,
        source_topic: String,
    ) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = DeleteTopicRewriteRuleRequest {
            cluster_name: config.cluster_name.clone(),
            action,
            source_topic,
        };
        placement_delete_topic_rewrite_rule(&self.client_pool, &config.placement_center, request)
            .await?;
        Ok(())
    }
}
