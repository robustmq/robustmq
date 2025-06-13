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

use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::placement::mqtt::call::{
    placement_delete_auto_subscribe_rule, placement_list_auto_subscribe_rule,
    placement_set_auto_subscribe_rule,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use protocol::placement_center::placement_center_mqtt::{
    DeleteAutoSubscribeRuleRequest, ListAutoSubscribeRuleRequest, SetAutoSubscribeRuleRequest,
};

use crate::handler::error::MqttBrokerError;

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
        let config = broker_mqtt_conf();
        let request = ListAutoSubscribeRuleRequest {
            cluster_name: config.cluster_name.clone(),
        };
        let reply = placement_list_auto_subscribe_rule(
            &self.client_pool,
            &config.placement_center,
            request,
        )
        .await?;
        let mut list = Vec::new();
        for raw in reply.auto_subscribe_rules {
            list.push(serde_json::from_slice::<MqttAutoSubscribeRule>(
                raw.as_slice(),
            )?);
        }
        Ok(list)
    }

    pub async fn set_auto_subscribe_rule(
        &self,
        auto_subscribe_rule: MqttAutoSubscribeRule,
    ) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = SetAutoSubscribeRuleRequest {
            cluster_name: config.cluster_name.clone(),
            topic: auto_subscribe_rule.topic.clone(),
            qos: Into::<u8>::into(auto_subscribe_rule.qos) as u32,
            no_local: auto_subscribe_rule.no_local,
            retain_as_published: auto_subscribe_rule.retain_as_published,
            retained_handling: Into::<u8>::into(auto_subscribe_rule.retained_handling) as u32,
        };
        placement_set_auto_subscribe_rule(&self.client_pool, &config.placement_center, request)
            .await?;
        Ok(())
    }

    pub async fn delete_auto_subscribe_rule(&self, topic: String) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = DeleteAutoSubscribeRuleRequest {
            cluster_name: config.cluster_name.clone(),
            topic,
        };
        placement_delete_auto_subscribe_rule(&self.client_pool, &config.placement_center, request)
            .await?;
        Ok(())
    }
}
