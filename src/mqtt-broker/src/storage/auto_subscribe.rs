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
    placement_delete_auto_subscribe_rule, placement_list_auto_subscribe_rule,
    placement_set_auto_subscribe_rule,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use protocol::meta::meta_service_mqtt::{
    DeleteAutoSubscribeRuleRequest, ListAutoSubscribeRuleRequest, SetAutoSubscribeRuleRequest,
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
    ) -> Result<Vec<MqttAutoSubscribeRule>, MqttBrokerError> {
        let config = broker_config();
        let request = ListAutoSubscribeRuleRequest {};
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

    pub async fn set_auto_subscribe_rule(
        &self,
        auto_subscribe_rule: MqttAutoSubscribeRule,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = SetAutoSubscribeRuleRequest {
            topic: auto_subscribe_rule.topic.clone(),
            qos: Into::<u8>::into(auto_subscribe_rule.qos) as u32,
            no_local: auto_subscribe_rule.no_local,
            retain_as_published: auto_subscribe_rule.retain_as_published,
            retained_handling: Into::<u8>::into(auto_subscribe_rule.retained_handling) as u32,
        };
        placement_set_auto_subscribe_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_auto_subscribe_rule(&self, topic: String) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteAutoSubscribeRuleRequest { topic };
        placement_delete_auto_subscribe_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }
}
