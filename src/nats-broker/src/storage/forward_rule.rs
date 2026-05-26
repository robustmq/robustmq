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
use common_config::broker::broker_config;
use grpc_clients::meta::mq9::call::{
    placement_create_mq9_forward_rule, placement_delete_mq9_forward_rule,
    placement_list_mq9_forward_rule, placement_update_mq9_forward_rule,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mq9::forward_rule::Mq9ForwardRule;
use protocol::meta::meta_service_mq9::{
    CreateForwardRuleRequest, DeleteForwardRuleRequest, ListForwardRuleRequest,
    UpdateForwardRuleRequest,
};
use std::sync::Arc;
use tonic::Streaming;

pub struct Mq9ForwardRuleStorage {
    client_pool: Arc<ClientPool>,
}

impl Mq9ForwardRuleStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        Mq9ForwardRuleStorage { client_pool }
    }

    pub async fn create(&self, rule: &Mq9ForwardRule) -> Result<(), CommonError> {
        let config = broker_config();
        let request = CreateForwardRuleRequest {
            tenant: rule.tenant.clone(),
            rule_name: rule.rule_name.clone(),
            content: rule.encode()?,
        };
        placement_create_mq9_forward_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn update(&self, rule: &Mq9ForwardRule) -> Result<(), CommonError> {
        let config = broker_config();
        let request = UpdateForwardRuleRequest {
            tenant: rule.tenant.clone(),
            rule_name: rule.rule_name.clone(),
            content: rule.encode()?,
        };
        placement_update_mq9_forward_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete(&self, tenant: &str, rule_name: &str) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteForwardRuleRequest {
            tenant: tenant.to_string(),
            rule_name: rule_name.to_string(),
        };
        placement_delete_mq9_forward_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn list(
        &self,
        tenant: &str,
        rule_name: &str,
    ) -> Result<Vec<Mq9ForwardRule>, CommonError> {
        let config = broker_config();
        let request = ListForwardRuleRequest {
            tenant: tenant.to_string(),
            rule_name: rule_name.to_string(),
        };
        let mut stream: Streaming<_> = placement_list_mq9_forward_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;

        let mut rules = Vec::new();
        while let Some(reply) = stream.message().await? {
            rules.push(Mq9ForwardRule::decode(&reply.rule)?);
        }
        Ok(rules)
    }

    pub async fn list_all(&self) -> Result<Vec<Mq9ForwardRule>, CommonError> {
        self.list("", "").await
    }
}
