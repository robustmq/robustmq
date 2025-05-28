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
    use std::sync::Arc;

    use grpc_clients::mqtt::admin::call::{
        mqtt_broker_create_topic_rewrite_rule, mqtt_broker_delete_topic_rewrite_rule,
    };
    use grpc_clients::pool::ClientPool;
    use protocol::broker_mqtt::broker_mqtt_admin::{
        CreateTopicRewriteRuleRequest, DeleteTopicRewriteRuleRequest,
    };

    #[tokio::test]
    async fn test_topic_rewrite_rule() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec!["localhost:9981".to_string()];

        let action: String = "All".to_string();
        let source_topic: String = "x/#".to_string();
        let dest_topic: String = "x/y/z/$1".to_string();
        let re: String = "^x/y/(.+)$".to_string();

        let req = CreateTopicRewriteRuleRequest {
            action: action.clone(),
            source_topic: source_topic.clone(),
            dest_topic: dest_topic.clone(),
            regex: re.clone(),
        };

        mqtt_broker_create_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();

        let req = DeleteTopicRewriteRuleRequest {
            action: action.clone(),
            source_topic: source_topic.clone(),
        };
        mqtt_broker_delete_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();
    }
}
