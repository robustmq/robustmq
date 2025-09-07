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

use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::mqtt::call::{
    placement_create_topic, placement_create_topic_rewrite_rule, placement_delete_topic,
    placement_delete_topic_rewrite_rule, placement_list_topic, placement_list_topic_rewrite_rule,
    placement_set_topic_retain_message,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::meta::placement_center_mqtt::{
    CreateTopicRequest, CreateTopicRewriteRuleRequest, DeleteTopicRequest,
    DeleteTopicRewriteRuleRequest, ListTopicRequest, ListTopicRewriteRuleRequest,
    SetTopicRetainMessageRequest,
};
use std::sync::Arc;

use crate::common::types::ResultMqttBrokerError;
use crate::handler::error::MqttBrokerError;

pub struct TopicStorage {
    client_pool: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        TopicStorage { client_pool }
    }

    pub async fn save_topic(&self, topic: MQTTTopic) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = CreateTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic.topic_name.clone(),
            content: topic.encode(),
        };
        placement_create_topic(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_topic(&self, topic_name: String) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
        };
        placement_delete_topic(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn all(&self) -> Result<DashMap<String, MQTTTopic>, MqttBrokerError> {
        let config = broker_config();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: "".to_string(),
        };
        let mut data_stream = placement_list_topic(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        let results = DashMap::with_capacity(2);

        while let Some(data) = data_stream.message().await? {
            let topic = serde_json::from_slice::<MQTTTopic>(&data.topic)?;
            results.insert(topic.topic_name.clone(), topic);
        }

        Ok(results)
    }

    pub async fn get_topic(&self, topic_name: &str) -> Result<Option<MQTTTopic>, MqttBrokerError> {
        let config = broker_config();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic_name.to_owned(),
        };

        let mut data_stream = placement_list_topic(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
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
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
            retain_message: retain_message.encode(),
            retain_message_expired_at,
        };
        placement_set_topic_retain_message(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_retain_message(&self, topic_name: String) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
            retain_message: Vec::new(),
            retain_message_expired_at: 0,
        };
        placement_set_topic_retain_message(
            &self.client_pool,
            &config.get_placement_center_addr(),
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

    pub async fn all_topic_rewrite_rule(
        &self,
    ) -> Result<Vec<MqttTopicRewriteRule>, MqttBrokerError> {
        let config = broker_config();
        let request = ListTopicRewriteRuleRequest {
            cluster_name: config.cluster_name.clone(),
        };
        let reply = placement_list_topic_rewrite_rule(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
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
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = CreateTopicRewriteRuleRequest {
            cluster_name: config.cluster_name.clone(),
            action: req.action.clone(),
            source_topic: req.source_topic.clone(),
            dest_topic: req.dest_topic.clone(),
            regex: req.regex.clone(),
        };
        placement_create_topic_rewrite_rule(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_topic_rewrite_rule(
        &self,
        action: String,
        source_topic: String,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteTopicRewriteRuleRequest {
            cluster_name: config.cluster_name.clone(),
            action,
            source_topic,
        };
        placement_delete_topic_rewrite_rule(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
    use common_base::tools::unique_id;
    use common_config::broker::{broker_config, default_broker_config, init_broker_conf_by_config};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::message::MqttMessage;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::{Publish, PublishProperties};

    use crate::storage::topic::TopicStorage;

    #[tokio::test]
    async fn topic_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_pool);

        let topic_name: String = "test_password".to_string();
        let topic = MQTTTopic::new(unique_id(), config.cluster_name.clone(), topic_name.clone());
        match topic_storage.save_topic(topic).await {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        }

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

        let mqtt_conf = broker_config();

        let topic = MQTTTopic::new(
            unique_id(),
            mqtt_conf.cluster_name.clone(),
            topic_name.clone(),
        );
        topic_storage.save_topic(topic).await.unwrap();

        let result = topic_storage.get_topic(&topic_name).await.unwrap().unwrap();
        println!("{result:?}");

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
