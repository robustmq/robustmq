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
use grpc_clients::meta::mqtt::call::{
    placement_get_topic_retain_message, placement_set_topic_retain_message,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
use protocol::meta::meta_service_mqtt::{
    GetTopicRetainMessageRequest, SetTopicRetainMessageRequest,
};
use std::sync::Arc;

pub struct RetainStorage {
    client_pool: Arc<ClientPool>,
}

impl RetainStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        RetainStorage { client_pool }
    }
    pub async fn set_retain_message(
        &self,
        tenant: &str,
        topic_name: &str,
        retain_message: &MQTTRetainMessage,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = SetTopicRetainMessageRequest {
            tenant: tenant.to_string(),
            topic_name: topic_name.to_string(),
            retain_message: Some(retain_message.encode()?.to_vec()),
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
            ..Default::default()
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
    ) -> Result<Option<MQTTRetainMessage>, MqttBrokerError> {
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
            let message = MQTTRetainMessage::decode(&data)?;
            return Ok(Some(message));
        }
        Ok(None)
    }
}
