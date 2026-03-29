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

use crate::core::error::MqttBrokerError;
use crate::core::tool::ResultMqttBrokerError;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::mqtt::call::{
    placement_create_topic, placement_create_topic_rewrite_rule, placement_delete_topic,
    placement_delete_topic_rewrite_rule, placement_get_topic_retain_message, placement_list_topic,
    placement_list_topic_rewrite_rule, placement_set_topic_retain_message,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::meta::meta_service_mqtt::{
    CreateTopicRequest, CreateTopicRewriteRuleRequest, DeleteTopicRequest,
    DeleteTopicRewriteRuleRequest, GetTopicRetainMessageRequest, ListTopicRequest,
    ListTopicRewriteRuleRequest, SetTopicRetainMessageRequest,
};
use std::sync::Arc;

pub struct TopicStorage {
    client_pool: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        TopicStorage { client_pool }
    }

    pub async fn create_topic(&self, topic: &Topic) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = CreateTopicRequest {
            tenant: topic.tenant.clone(),
            topic_name: topic.topic_name.clone(),
            content: topic.encode()?,
        };

        placement_create_topic(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete_topic(&self, tenant: &str, topic_name: &str) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteTopicRequest {
            tenant: tenant.to_string(),
            topic_name: topic_name.to_string(),
        };
        placement_delete_topic(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn all(&self) -> Result<DashMap<String, Topic>, MqttBrokerError> {
        let config = broker_config();
        let request = ListTopicRequest {
            ..Default::default()
        };
        let mut data_stream =
            placement_list_topic(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        let results = DashMap::with_capacity(2);

        while let Some(data) = data_stream.message().await? {
            let topic = Topic::decode(&data.topic)?;
            results.insert(topic.topic_name.clone(), topic);
        }

        Ok(results)
    }

    pub async fn get_topic(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<Option<Topic>, MqttBrokerError> {
        let config = broker_config();
        let request = ListTopicRequest {
            tenant: tenant.to_owned(),
            topic_name: topic_name.to_owned(),
        };

        let mut data_stream =
            placement_list_topic(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        if let Some(data) = data_stream.message().await? {
            let topic = Topic::decode(&data.topic)?;
            return Ok(Some(topic));
        }

        Ok(None)
    }

    // retain message
    pub async fn set_retain_message(
        &self,
        tenant: &str,
        topic_name: &str,
        retain_message: &MqttRecordMeta,
        retain_message_expired_at: u64,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = SetTopicRetainMessageRequest {
            tenant: tenant.to_string(),
            topic_name: topic_name.to_string(),
            retain_message: Some(retain_message.encode()?.to_vec()),
            retain_message_expired_at,
        };
        placement_set_topic_retain_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_retain_message(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = SetTopicRetainMessageRequest {
            tenant: tenant.to_string(),
            topic_name: topic_name.to_owned(),
            retain_message: None,
            retain_message_expired_at: 0,
        };
        placement_set_topic_retain_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn get_retain_message(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<(Option<MqttRecordMeta>, Option<u64>), MqttBrokerError> {
        let config = broker_config();
        let request = GetTopicRetainMessageRequest {
            tenant: tenant.to_owned(),
            topic_name: topic_name.to_owned(),
        };

        let reply = placement_get_topic_retain_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;

        if let Some(data) = reply.retain_message {
            Ok((
                Some(MqttRecordMeta::decode(&data)?),
                Some(reply.retain_message_expired_at),
            ))
        } else {
            Ok((None, None))
        }
    }

    pub async fn all_topic_rewrite_rule(
        &self,
    ) -> Result<Vec<MqttTopicRewriteRule>, MqttBrokerError> {
        self.topic_rewrite_rule_by_tenant("").await
    }

    pub async fn topic_rewrite_rule_by_tenant(
        &self,
        tenant: &str,
    ) -> Result<Vec<MqttTopicRewriteRule>, MqttBrokerError> {
        let config = broker_config();
        let request = ListTopicRewriteRuleRequest {
            tenant: tenant.to_string(),
        };
        let reply = placement_list_topic_rewrite_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        let mut results = Vec::with_capacity(8);
        for raw in reply.topic_rewrite_rules {
            let data = MqttTopicRewriteRule::decode(&raw)?;
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
            name: req.name.clone(),
            desc: req.desc.clone(),
            tenant: req.tenant.clone(),
            action: req.action.clone(),
            source_topic: req.source_topic.clone(),
            dest_topic: req.dest_topic.clone(),
            regex: req.regex.clone(),
        };
        placement_create_topic_rewrite_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_topic_rewrite_rule(
        &self,
        tenant: String,
        name: String,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteTopicRewriteRuleRequest { tenant, name };
        placement_delete_topic_rewrite_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }
}
