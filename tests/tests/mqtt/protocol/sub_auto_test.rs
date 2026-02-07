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
            broker_addr_by_type, connect_server, create_test_env, distinct_conn, publish_data,
        },
        ClientTestProperties,
    };
    use admin_server::client::AdminHttpClient;
    use admin_server::mqtt::subscribe::{CreateAutoSubscribeReq, DeleteAutoSubscribeReq};
    use common_base::uuid::unique_id;
    use paho_mqtt::{Client, Message, Receiver, QOS_1};
    use std::time::Duration;

    struct AutoSubscribeTestContext<'a> {
        admin_client: &'a AdminHttpClient,
        topic_pattern: String,
        qos: u32,
    }

    impl<'a> AutoSubscribeTestContext<'a> {
        fn new(admin_client: &'a AdminHttpClient, topic_pattern: String, qos: u32) -> Self {
            Self {
                admin_client,
                topic_pattern,
                qos,
            }
        }

        async fn setup(&self) {
            create_auto_subscribe_rule(self.admin_client, &self.topic_pattern, self.qos).await;
        }

        async fn cleanup(&self) {
            delete_auto_subscribe_rule(self.admin_client, &self.topic_pattern).await;
        }
    }

    fn setup_client_and_consume(
        client_id: &str,
        username: Option<String>,
        password: Option<String>,
    ) -> (Client, Receiver<Option<Message>>) {
        let (user_name, pwd) = match (username, password) {
            (Some(u), Some(p)) => (u, p),
            (Some(u), None) => (u, String::new()),
            (None, Some(p)) => (String::new(), p),
            (None, None) => (String::new(), String::new()),
        };

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type("tcp"),
            user_name,
            password: pwd,
            ..Default::default()
        };

        let cli = connect_server(&client_properties);
        let rx = cli.start_consuming();
        (cli, rx)
    }

    fn verify_message_received(
        rx: &Receiver<Option<Message>>,
        expected_content: &str,
        expected_topic: &str,
        test_name: &str,
    ) {
        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(Some(msg)) => {
                    if msg.topic() == expected_topic && msg.payload_str() == expected_content {
                        return;
                    }
                }
                Ok(None) => {}
                Err(_) => {
                    if start.elapsed() >= timeout {
                        break;
                    }
                }
            }
        }

        panic!(
            "{}: Did not receive expected message (topic: '{}', content: '{}') within timeout",
            test_name, expected_topic, expected_content
        );
    }

    async fn test_auto_subscribe_single_message(
        topic_pattern: &str,
        actual_topic: &str,
        client_id: &str,
        username: Option<String>,
        password: Option<String>,
        message_content: &str,
        test_name: &str,
    ) {
        let admin_client = create_test_env().await;
        let ctx = AutoSubscribeTestContext::new(&admin_client, topic_pattern.to_string(), 1);
        ctx.setup().await;

        let (cli, rx) = setup_client_and_consume(client_id, username, password);

        let msg = Message::new(actual_topic, message_content, QOS_1);
        publish_data(&cli, msg, false);

        verify_message_received(&rx, message_content, actual_topic, test_name);

        distinct_conn(cli);
        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn sub_auto_basic_test() {
        let uniq = unique_id();
        let topic = format!("/sub_auto_test/basic/{uniq}");
        let client_id = format!("sub_auto_basic_{}", unique_id());

        test_auto_subscribe_single_message(
            &topic,
            &topic,
            &client_id,
            None,
            None,
            "sub_auto_test mqtt message",
            "sub_auto_basic_test",
        )
        .await;
    }

    #[tokio::test]
    async fn sub_auto_clientid_placeholder_test() {
        let uniq = unique_id();
        let topic_pattern = format!("client/${{clientid}}/cmd/{uniq}");
        let client_id = format!("device_{}", unique_id());
        let actual_topic = format!("client/{}/cmd/{uniq}", client_id);

        test_auto_subscribe_single_message(
            &topic_pattern,
            &actual_topic,
            &client_id,
            None,
            None,
            "command for specific client",
            "sub_auto_clientid_placeholder_test",
        )
        .await;
    }

    #[tokio::test]
    async fn sub_auto_username_placeholder_test() {
        let uniq = unique_id();
        let topic_pattern = format!("user/${{username}}/inbox/{uniq}");
        let username = "admin".to_string();
        let password = "robustmq".to_string();
        let client_id = format!("sub_auto_username_{}", unique_id());
        let actual_topic = format!("user/{}/inbox/{uniq}", username);

        test_auto_subscribe_single_message(
            &topic_pattern,
            &actual_topic,
            &client_id,
            Some(username),
            Some(password),
            "message for specific user",
            "sub_auto_username_placeholder_test",
        )
        .await;
    }

    #[tokio::test]
    async fn sub_auto_empty_username_test() {
        let uniq = unique_id();
        let topic_pattern = format!("user/${{username}}/msg/{uniq}");
        let client_id = format!("sub_auto_no_username_{}", unique_id());
        let actual_topic = format!("user/admin/msg/{uniq}");

        test_auto_subscribe_single_message(
            &topic_pattern,
            &actual_topic,
            &client_id,
            None,
            None,
            "message with default username",
            "sub_auto_empty_username_test",
        )
        .await;
    }

    #[tokio::test]
    async fn sub_auto_multiple_placeholders_test() {
        let uniq = unique_id();
        let topic_pattern = format!("device/${{clientid}}/user/${{username}}/{uniq}");
        let client_id = format!("dev_{}", unique_id());
        let username = "admin".to_string();
        let password = "robustmq".to_string();
        let actual_topic = format!("device/{}/user/{}/{uniq}", client_id, username);

        test_auto_subscribe_single_message(
            &topic_pattern,
            &actual_topic,
            &client_id,
            Some(username),
            Some(password),
            "message with multiple placeholders",
            "sub_auto_multiple_placeholders_test",
        )
        .await;
    }

    #[tokio::test]
    async fn sub_auto_multiple_rules_test() {
        let admin_client = create_test_env().await;
        let uniq = unique_id();
        let topics = vec![
            format!("topic1/{uniq}"),
            format!("topic2/{uniq}"),
            format!("topic3/{uniq}"),
        ];

        for topic in &topics {
            create_auto_subscribe_rule(&admin_client, topic, 1).await;
        }

        let client_id = format!("sub_auto_multiple_{}", unique_id());
        let (cli, rx) = setup_client_and_consume(&client_id, None, None);

        for (i, topic) in topics.iter().enumerate() {
            let msg = Message::new(topic, format!("message {}", i + 1), QOS_1);
            publish_data(&cli, msg, false);
        }

        let mut received_topics = std::collections::HashSet::new();
        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();

        while received_topics.len() < topics.len() && start.elapsed() < timeout {
            if let Ok(Some(msg)) = rx.recv_timeout(Duration::from_secs(1)) {
                let topic = msg.topic().to_string();
                if topics.contains(&topic) {
                    received_topics.insert(topic);
                }
            }
        }

        assert_eq!(
            received_topics.len(),
            topics.len(),
            "Should receive all messages. Got: {:?}, Expected: {:?}",
            received_topics,
            topics
        );
        for topic in &topics {
            assert!(
                received_topics.contains(topic),
                "Should receive message from topic: {}",
                topic
            );
        }

        distinct_conn(cli);

        for topic in &topics {
            delete_auto_subscribe_rule(&admin_client, topic).await;
        }
    }

    #[tokio::test]
    async fn sub_auto_different_qos_test() {
        let admin_client = create_test_env().await;
        let uniq = unique_id();
        let qos_configs = vec![
            (format!("qos0/{uniq}"), 0, "qos 0 message"),
            (format!("qos1/{uniq}"), 1, "qos 1 message"),
            (format!("qos2/{uniq}"), 2, "qos 2 message"),
        ];

        for (topic, qos, _) in &qos_configs {
            create_auto_subscribe_rule(&admin_client, topic, *qos).await;
        }

        let client_id = format!("sub_auto_qos_{}", unique_id());
        let (cli, rx) = setup_client_and_consume(&client_id, None, None);

        for (topic, qos, content) in &qos_configs {
            let msg = Message::new(topic, *content, *qos as i32);
            publish_data(&cli, msg, false);
        }

        let mut received_count = 0;
        for _ in 0..qos_configs.len() {
            if rx.recv_timeout(Duration::from_secs(3)).is_ok() {
                received_count += 1;
            }
        }

        assert_eq!(
            received_count,
            qos_configs.len(),
            "Should receive messages from all QoS levels"
        );

        distinct_conn(cli);

        for (topic, _, _) in &qos_configs {
            delete_auto_subscribe_rule(&admin_client, topic).await;
        }
    }

    async fn create_auto_subscribe_rule(admin_client: &AdminHttpClient, topic: &str, qos: u32) {
        let request = CreateAutoSubscribeReq {
            topic: topic.to_string(),
            qos,
            no_local: false,
            retain_as_published: false,
            retained_handling: 0,
        };
        let res = admin_client.create_auto_subscribe(&request).await;
        assert!(
            res.is_ok(),
            "Failed to create auto-subscribe rule for topic: {}",
            topic
        );
    }

    async fn delete_auto_subscribe_rule(admin_client: &AdminHttpClient, topic: &str) {
        let request = DeleteAutoSubscribeReq {
            topic_name: topic.to_string(),
        };
        let res = admin_client.delete_auto_subscribe(&request).await;
        assert!(
            res.is_ok(),
            "Failed to delete auto-subscribe rule for topic: {}",
            topic
        );
    }
}
