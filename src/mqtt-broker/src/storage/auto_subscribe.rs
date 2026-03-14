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

use common_config::broker::broker_config;
use grpc_clients::meta::mqtt::call::{
    placement_create_auto_subscribe_rule, placement_delete_auto_subscribe_rule,
    placement_list_auto_subscribe_rule,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use protocol::meta::meta_service_mqtt::{
    CreateAutoSubscribeRuleRequest, DeleteAutoSubscribeRuleRequest, ListAutoSubscribeRuleRequest,
};

use crate::core::error::MqttBrokerError;
use crate::core::tool::ResultMqttBrokerError;

pub struct AutoSubscribeStorage {
    client_pool: Arc<ClientPool>,
}

impl AutoSubscribeStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        AutoSubscribeStorage { client_pool }
    }

    pub async fn list_auto_subscribe_rule(
        &self,
        tenant: Option<String>,
    ) -> Result<Vec<MqttAutoSubscribeRule>, MqttBrokerError> {
        let config = broker_config();
        let request = ListAutoSubscribeRuleRequest {
            tenant: tenant.unwrap_or_default(),
        };
        let reply = placement_list_auto_subscribe_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        let mut list = Vec::new();
        for raw in reply.auto_subscribe_rules {
            list.push(MqttAutoSubscribeRule::decode(&raw)?);
        }
        Ok(list)
    }

    pub async fn create_auto_subscribe_rule(
        &self,
        auto_subscribe_rule: MqttAutoSubscribeRule,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let content = auto_subscribe_rule
            .encode()
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
        let request = CreateAutoSubscribeRuleRequest { content };
        placement_create_auto_subscribe_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_auto_subscribe_rule(
        &self,
        tenant: String,
        topic: String,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteAutoSubscribeRuleRequest { tenant, topic };
        placement_delete_auto_subscribe_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }
}
