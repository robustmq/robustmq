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
    use grpc_clients::mqtt::admin::call::mqtt_broker_create_topic_rewrite_rule;
    use grpc_clients::pool::ClientPool;
    use paho_mqtt::{Message, QOS_1};
    use protocol::broker_mqtt::broker_mqtt_admin::CreateTopicRewriteRuleRequest;

    use crate::mqtt_protocol::common::{
        broker_addr, build_client_id, connect_server5, distinct_conn,
    };

    #[tokio::test]
    async fn client5_rewrite_pub_test() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec!["127.0.0.1:9981".to_string()];

        let action: String = "All".to_string();
        let source_topic: String = "/tests_r1/#".to_string();
        let dest_topic: String = "/test_rewrite/$1".to_string();
        let re: String = "^/tests_r1/(.+)$".to_string();

        let req = CreateTopicRewriteRuleRequest {
            action: action.clone(),
            source_topic: source_topic.clone(),
            dest_topic: dest_topic.clone(),
            regex: re.clone(),
        };

        mqtt_broker_create_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();

        let sub_qos = &[0];
        let uuid = unique_id();
        let topic = format!("/tests_r1/{}", uuid);
        let rewrite_topic = format!("/test_rewrite/{}", uuid);
        let sub_topic = rewrite_topic;
        simple_test(
            topic.clone(),
            sub_topic.clone(),
            sub_qos,
            "2".to_string(),
            String::new(),
        )
        .await;
    }

    #[tokio::test]
    async fn client5_rewrite_sub_test() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec!["127.0.0.1:9981".to_string()];

        let action: String = "Subscribe".to_string();
        let source_topic: String = "/tests_r2/#".to_string();
        let dest_topic: String = "/testsub/$1".to_string();
        let re: String = "^/tests_r2/(.+)$".to_string();

        let req = CreateTopicRewriteRuleRequest {
            action: action.clone(),
            source_topic: source_topic.clone(),
            dest_topic: dest_topic.clone(),
            regex: re.clone(),
        };

        mqtt_broker_create_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();

        let sub_qos = &[0];
        let uuid = unique_id();
        let topic = format!("/testsub/{}", uuid);
        let rewrite_topic = format!("/tests_r2/{}", uuid);
        let sub_topic = rewrite_topic;
        simple_test(
            topic.clone(),
            sub_topic.clone(),
            sub_qos,
            "2".to_string(),
            String::new(),
        )
        .await;
    }

    #[tokio::test]
    async fn client5_rewrite_unsub_test() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec!["127.0.0.1:9981".to_string()];

        let action: String = "Subscribe".to_string();
        let source_topic: String = "/testUnsub/#".to_string();
        let dest_topic: String = "/test_r3/$1".to_string();
        let re: String = "^/testUnsub/(.+)$".to_string();

        let req = CreateTopicRewriteRuleRequest {
            action: action.clone(),
            source_topic: source_topic.clone(),
            dest_topic: dest_topic.clone(),
            regex: re.clone(),
        };

        mqtt_broker_create_topic_rewrite_rule(&client_pool, &addrs, req)
            .await
            .unwrap();

        let sub_qos = &[0];
        let uuid = unique_id();
        let topic = format!("/test_r3/{}", uuid);
        let sub_topic = topic.clone();
        simple_test(
            topic.clone(),
            sub_topic.clone(),
            sub_qos,
            "2".to_string(),
            topic,
        )
        .await;
    }

    async fn simple_test(
        pub_topic: String,
        sub_topic: String,
        sub_qos: &[i32],
        payload_flag: String,
        unsub_topic: String,
    ) {
        let client_id = build_client_id("topic_rewrite_rule_test");
        let addr = broker_addr();
        let sub_topics = &[sub_topic.clone()];

        let cli = connect_server5(&client_id, &addr, false, false);
        let message_content = format!("mqtt {payload_flag} message");

        // Publish
        let msg = Message::new(pub_topic.clone(), message_content.clone(), QOS_1);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
        // Subscribe
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }
        if let Some(msg) = rx.iter().next() {
            let msg = msg.unwrap();
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            assert_eq!(payload, message_content);
        }
        if !unsub_topic.is_empty() {
            cli.unsubscribe_many(&[unsub_topic]).unwrap();
        }
        distinct_conn(cli);
    }
}
