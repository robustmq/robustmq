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
    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
            publish_data, subscribe_data_by_qos,
        },
        ClientTestProperties,
    };
    use admin_server::mqtt::topic::{CreateTopicRewriteReq, DeleteTopicRewriteReq};
    use common_base::uuid::unique_id;
    use paho_mqtt::{Message, MessageBuilder};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn emqx_example_rule1_match_test() {
        let prefix = format!("emqx_ex1_{}", unique_id());
        let source_topic = format!("{prefix}_y/a/z/b");
        let expected_topic = format!("{prefix}_y/z/b");

        let rules = vec![CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_y/+/z/#"),
            dest_topic: format!("{prefix}_y/z/$2"),
            regex: format!("^{prefix}_y/(.+)/z/(.+)$"),
        }];

        test_topic_rewrite_with_rules(&prefix, &source_topic, &expected_topic, rules).await;
    }

    #[tokio::test]
    async fn emqx_example_rule1_no_match_test() {
        let prefix = format!("emqx_ex2_{}", unique_id());
        let source_topic = format!("{prefix}_y/def");
        let expected_topic = format!("{prefix}_y/def");

        let rules = vec![CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_y/+/z/#"),
            dest_topic: format!("{prefix}_y/z/$2"),
            regex: format!("^{prefix}_y/(.+)/z/(.+)$"),
        }];

        test_topic_rewrite_with_rules(&prefix, &source_topic, &expected_topic, rules).await;
    }

    #[tokio::test]
    async fn emqx_example_rule2_no_match_regex_test() {
        let prefix = format!("emqx_ex3_{}", unique_id());
        let source_topic = format!("{prefix}_x/1/2");
        let expected_topic = format!("{prefix}_x/1/2");

        let rules = vec![CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_x/#"),
            dest_topic: format!("{prefix}_z/y/x/$1"),
            regex: format!("^{prefix}_x/y/(.+)$"),
        }];

        test_topic_rewrite_with_rules(&prefix, &source_topic, &expected_topic, rules).await;
    }

    #[tokio::test]
    async fn emqx_example_rule3_priority_test() {
        let prefix = format!("emqx_ex4_{}", unique_id());
        let source_topic = format!("{prefix}_x/y/2");
        let expected_topic = format!("{prefix}_z/y/2");

        let rules = vec![
            CreateTopicRewriteReq {
                action: "All".to_string(),
                source_topic: format!("{prefix}_x/y/+"),
                dest_topic: format!("{prefix}_z/y/$1"),
                regex: format!("^{prefix}_x/y/(\\d+)$"),
            },
            CreateTopicRewriteReq {
                action: "All".to_string(),
                source_topic: format!("{prefix}_x/#"),
                dest_topic: format!("{prefix}_z/y/x/$1"),
                regex: format!("^{prefix}_x/y/(.+)$"),
            },
        ];

        test_topic_rewrite_with_rules(&prefix, &source_topic, &expected_topic, rules).await;
    }

    #[tokio::test]
    async fn emqx_example_rule3_no_match_regex_test() {
        let prefix = format!("emqx_ex5_{}", unique_id());
        let source_topic = format!("{prefix}_x/y/z");
        let expected_topic = format!("{prefix}_x/y/z");

        let rules = vec![
            CreateTopicRewriteReq {
                action: "All".to_string(),
                source_topic: format!("{prefix}_x/y/+"),
                dest_topic: format!("{prefix}_z/y/$1"),
                regex: format!("^{prefix}_x/y/(\\d+)$"),
            },
            CreateTopicRewriteReq {
                action: "All".to_string(),
                source_topic: format!("{prefix}_x/#"),
                dest_topic: format!("{prefix}_z/y/x/$1"),
                regex: format!("^{prefix}_x/y/(.+)$"),
            },
        ];

        test_topic_rewrite_with_rules(&prefix, &source_topic, &expected_topic, rules).await;
    }

    async fn test_topic_rewrite_with_rules(
        prefix: &str,
        source_topic: &str,
        expected_topic: &str,
        rules: Vec<CreateTopicRewriteReq>,
    ) {
        let admin_client = create_test_env().await;
        let network = "tcp";
        let protocol = 5;
        let qos = 1;

        let init_client_id = build_client_id(&format!("{prefix}_init"));
        let init_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: init_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let init_cli = connect_server(&init_properties);
        let msg = MessageBuilder::new()
            .payload("init topic".to_string())
            .topic(source_topic.to_string())
            .qos(qos)
            .finalize();
        publish_data(&init_cli, msg, false);
        distinct_conn(init_cli);

        for (i, rule) in rules.iter().enumerate() {
            admin_client.create_topic_rewrite(rule).await.unwrap();
            if i < rules.len() - 1 {
                sleep(Duration::from_millis(200)).await;
            }
        }

        sleep(Duration::from_secs(5)).await;

        let client_id = build_client_id(&format!("{prefix}_test"));
        let properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&properties);

        let message = format!("message for {}", source_topic);
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(source_topic.to_string())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        sleep(Duration::from_secs(2)).await;

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message
        };
        let res = subscribe_data_by_qos(&cli, expected_topic, qos, call_fn);
        assert!(
            res.is_ok(),
            "Failed: {} -> {}",
            source_topic,
            expected_topic
        );
        distinct_conn(cli);

        for rule in rules.iter() {
            let del_req = DeleteTopicRewriteReq {
                action: rule.action.clone(),
                source_topic: rule.source_topic.clone(),
            };
            admin_client.delete_topic_rewrite(&del_req).await.unwrap();
        }
    }
}
