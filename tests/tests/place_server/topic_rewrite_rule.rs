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

    use common_base::tools::unique_id;
    use grpc_clients::placement::mqtt::call::{
        placement_create_topic_rewrite_rule, placement_delete_topic_rewrite_rule,
        placement_list_topic_rewrite_rule,
    };
    use grpc_clients::pool::ClientPool;
    use protocol::placement_center::placement_center_mqtt::{
        CreateTopicRewriteRuleRequest, DeleteTopicRewriteRuleRequest, ListTopicRewriteRuleRequest,
    };

    #[tokio::test]
    async fn test_topic_rewrite_rule() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec!["localhost:1228".to_string()];

        let cluster_name = unique_id();
        let action: String = "All".to_string();
        let source_topic: String = "x/#".to_string();
        let dest_topic: String = "x/y/z/$1".to_string();
        let re: String = "^x/y/(.+)$".to_string();

        let req = CreateTopicRewriteRuleRequest {
            cluster_name: cluster_name.clone(),
            action: action.clone(),
            source_topic: source_topic.clone(),
            dest_topic: dest_topic.clone(),
            regex: re.clone(),
        };

        placement_create_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();

        let req = ListTopicRewriteRuleRequest {
            cluster_name: cluster_name.clone(),
        };
        let resp = placement_list_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();
        assert_eq!(resp.topic_rewrite_rules.len(), 1);

        let req = DeleteTopicRewriteRuleRequest {
            cluster_name: cluster_name.clone(),
            action: action.clone(),
            source_topic: source_topic.clone(),
        };
        placement_delete_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();

        let req = ListTopicRewriteRuleRequest {
            cluster_name: cluster_name.clone(),
        };
        let resp = placement_list_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();
        assert_eq!(resp.topic_rewrite_rules.len(), 0);
    }
}
