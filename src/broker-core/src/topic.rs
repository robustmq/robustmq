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

use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::mqtt::call::{
    placement_create_topic, placement_delete_topic, placement_list_topic,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::Topic;
use protocol::meta::meta_service_mqtt::{CreateTopicRequest, DeleteTopicRequest, ListTopicRequest};
use std::sync::Arc;

pub struct TopicStorage {
    client_pool: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        TopicStorage { client_pool }
    }

    pub async fn create_topic(&self, topic: &Topic) -> ResultCommonError {
        let config = broker_config();
        let request = CreateTopicRequest {
            tenant: topic.tenant.clone(),
            topic_name: topic.topic_name.clone(),
            content: topic.encode()?,
        };

        placement_create_topic(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete_topic(&self, tenant: &str, topic_name: &str) -> ResultCommonError {
        let config = broker_config();
        let request = DeleteTopicRequest {
            tenant: tenant.to_string(),
            topic_name: topic_name.to_string(),
        };
        placement_delete_topic(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn all(&self) -> Result<DashMap<String, Topic>, CommonError> {
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
    ) -> Result<Option<Topic>, CommonError> {
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
}
