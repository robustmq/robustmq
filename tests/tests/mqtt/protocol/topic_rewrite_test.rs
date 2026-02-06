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
    use admin_server::mqtt::topic::CreateTopicRewriteReq;
    use common_base::uuid::unique_id;
    use paho_mqtt::{Message, MessageBuilder};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn pub_sub_rewrite_test() {
        let admin_client = create_test_env().await;

        let prefix = format!("pub_sub_rewrite_test_{}", unique_id());
        let source_topic = format!("{prefix}_y/a/z/b");
        let rewrite_topic = format!("{prefix}_y/z/b");
        println!("source_topic:{:?}", source_topic);
        println!("rewrite_topic:{:?}", rewrite_topic);

        let network = "tcp";
        let protocol = 5;
        let qos = 1;

        // create topic（write source_topic）
        let pub_client_id = build_client_id(&format!("{prefix}_pub"));
        let pub_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: pub_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let pub_cli = connect_server(&pub_properties);

        let message = "pub_sub_rewrite_test create topic message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(source_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&pub_cli, msg, false);

        // create topic rewrite rule
        let req = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_y/+/z/#"),
            dest_topic: format!("{prefix}_y/z/$2"),
            regex: format!("^{prefix}_y/(.+)/z/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req).await.unwrap();

        // write rewrite topic
        sleep(Duration::from_secs(5)).await;

        // publish data（write rewrite_topic)
        let message = "pub_sub_rewrite_test p1 message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(source_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&pub_cli, msg, false);

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message
        };
        let res = subscribe_data_by_qos(&pub_cli, &rewrite_topic, qos, call_fn);
        assert!(res.is_ok());
        distinct_conn(pub_cli);

        // Test 2: Subscribe to source, publish to rewritten topic
        let pub2_client_id = build_client_id(&format!("{prefix}_pub2"));
        let pub2_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: pub2_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let pub2_cli = connect_server(&pub2_properties);
        let message = "pub_sub_rewrite_test p2 message".to_string();
        let msg2 = MessageBuilder::new()
            .payload(message.clone())
            .topic(rewrite_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&pub2_cli, msg2, false);

        let call_fn2 = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message
        };
        let res = subscribe_data_by_qos(&pub2_cli, &source_topic, qos, call_fn2);
        assert!(res.is_ok());
        distinct_conn(pub2_cli);
    }

    #[tokio::test]
    async fn un_sub_rewrite_test() {
        let admin_client = create_test_env().await;

        let prefix = format!("unsub_rewrite_test_{}", unique_id());
        let source_topic = format!("{prefix}_y/a/z/b");

        let req = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_y/+/z/#"),
            dest_topic: format!("{prefix}_y/z/$2"),
            regex: format!("^{prefix}_y/(.+)/z/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req).await.unwrap();

        sleep(Duration::from_secs(5)).await;

        let network = "tcp";
        let protocol = 5;
        let qos = 1;

        let client_id = build_client_id(&prefix);
        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        cli.subscribe(&source_topic, qos).unwrap();
        sleep(Duration::from_secs(1)).await;
        cli.unsubscribe(&source_topic).unwrap();

        distinct_conn(cli);
    }

    #[tokio::test]
    async fn emqx_example_test() {
        let admin_client = create_test_env().await;
        let prefix = format!("emqx_example_{}", unique_id());

        let network = "tcp";
        let protocol = 5;
        let qos = 1;

        let test_cases = vec![
            (format!("{prefix}_y/a/z/b"), format!("{prefix}_y/z/b")),
            (format!("{prefix}_y/def"), format!("{prefix}_y/def")),
            (format!("{prefix}_x/1/2"), format!("{prefix}_x/1/2")),
            (format!("{prefix}_x/y/2"), format!("{prefix}_z/y/2")),
            (format!("{prefix}_x/y/z"), format!("{prefix}_x/y/z")),
        ];

        let init_client_id = build_client_id(&format!("{prefix}_init"));
        let init_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: init_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let init_cli = connect_server(&init_properties);

        for (source, _) in &test_cases {
            let msg = MessageBuilder::new()
                .payload("init topic".to_string())
                .topic(source.clone())
                .qos(qos)
                .finalize();
            publish_data(&init_cli, msg, false);
        }
        distinct_conn(init_cli);

        let req1 = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_y/+/z/#"),
            dest_topic: format!("{prefix}_y/z/$2"),
            regex: format!("^{prefix}_y/(.+)/z/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req1).await.unwrap();
        sleep(Duration::from_millis(200)).await;

        let req3 = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_x/y/+"),
            dest_topic: format!("{prefix}_z/y/$1"),
            regex: format!("^{prefix}_x/y/(\\d+)$"),
        };
        admin_client.create_topic_rewrite(&req3).await.unwrap();
        sleep(Duration::from_millis(200)).await;

        let req2 = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_x/#"),
            dest_topic: format!("{prefix}_z/y/x/$1"),
            regex: format!("^{prefix}_x/y/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req2).await.unwrap();
        sleep(Duration::from_millis(200)).await;

        sleep(Duration::from_secs(5)).await;

        for (source_topic, expected_topic) in test_cases {
            println!("source_topic:{}", source_topic);
            println!("expected_topic:{}", expected_topic);
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
                .topic(source_topic.clone())
                .qos(qos)
                .finalize();
            publish_data(&cli, msg, false);

            let call_fn = |msg: Message| {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                payload == message
            };
            let res = subscribe_data_by_qos(&cli, &expected_topic, qos, call_fn);
            assert!(
                res.is_ok(),
                "Failed: {} -> {}",
                source_topic,
                expected_topic
            );
            distinct_conn(cli);
        }
    }

    #[tokio::test]
    async fn action_publish_only_test() {
        let admin_client = create_test_env().await;

        let prefix = format!("action_publish_{}", unique_id());
        let source_topic = format!("{prefix}_test/topic");
        let rewrite_topic = format!("{prefix}_rewritten/topic");

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
            .topic(source_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&init_cli, msg, false);
        distinct_conn(init_cli);

        let req = CreateTopicRewriteReq {
            action: "Publish".to_string(),
            source_topic: format!("{prefix}_test/#"),
            dest_topic: format!("{prefix}_rewritten/$1"),
            regex: format!("^{prefix}_test/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req).await.unwrap();

        sleep(Duration::from_secs(5)).await;

        let client_id = build_client_id(&format!("{prefix}_test"));
        let properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&properties);

        let message = "action_publish_only_test message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(source_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message
        };
        let res = subscribe_data_by_qos(&cli, &rewrite_topic, qos, call_fn);
        assert!(res.is_ok());
        distinct_conn(cli);
    }
}
