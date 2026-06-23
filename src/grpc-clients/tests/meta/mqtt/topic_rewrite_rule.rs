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

#[cfg(test)]
mod tests {
    use crate::common::wait_until;
    use common_base::uuid::unique_id;
    use grpc_clients::meta::mqtt::call::{
        placement_create_topic_rewrite_rule, placement_delete_topic_rewrite_rule,
        placement_list_topic_rewrite_rule,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use protocol::meta::meta_service_mqtt::{
        CreateTopicRewriteRuleRequest, DeleteTopicRewriteRuleRequest, ListTopicRewriteRuleRequest,
    };
    use std::sync::Arc;

    fn has_rule(raws: &[Vec<u8>], name: &str) -> bool {
        raws.iter().any(|raw| {
            MqttTopicRewriteRule::decode(raw)
                .map(|r| r.name == name)
                .unwrap_or(false)
        })
    }

    #[tokio::test]
    async fn test_topic_rewrite_rule() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec!["127.0.0.1:1228".to_string()];
        let rule_name = unique_id();
        let action: String = "All".to_string();
        let source_topic: String = "x/#".to_string();
        let dest_topic: String = "x/y/z/$1".to_string();
        let re: String = "^x/y/(.+)$".to_string();

        let req = CreateTopicRewriteRuleRequest {
            name: rule_name.clone(),
            desc: String::new(),
            tenant: DEFAULT_TENANT.to_string(),
            action: action.clone(),
            source_topic: source_topic.clone(),
            dest_topic: dest_topic.clone(),
            regex: re.clone(),
        };

        placement_create_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();

        let present = wait_until(|| async {
            let req = ListTopicRewriteRuleRequest {
                tenant: DEFAULT_TENANT.to_string(),
            };
            match placement_list_topic_rewrite_rule(&client_pool, &addrs, req).await {
                Ok(resp) => has_rule(&resp.topic_rewrite_rules, &rule_name),
                Err(_) => false,
            }
        })
        .await;
        assert!(present, "created rule {rule_name} not visible");

        let req = DeleteTopicRewriteRuleRequest {
            tenant: DEFAULT_TENANT.to_string(),
            name: rule_name.clone(),
        };
        placement_delete_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();

        let absent = wait_until(|| async {
            let req = ListTopicRewriteRuleRequest {
                tenant: DEFAULT_TENANT.to_string(),
            };
            match placement_list_topic_rewrite_rule(&client_pool, &addrs, req).await {
                Ok(resp) => !has_rule(&resp.topic_rewrite_rules, &rule_name),
                Err(_) => false,
            }
        })
        .await;
        assert!(absent, "deleted rule {rule_name} still visible");
    }
}
