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
    use std::time::Duration;

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
            publish_data, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };
    use admin_server::mqtt::topic::CreateTopicRewriteReq;
    use common_base::uuid::unique_id;
    use paho_mqtt::{Message, MessageBuilder};
    use tokio::time::sleep;

    #[tokio::test]
    async fn pub_sub_rewrite_test() {
        let admin_client = create_test_env().await;

        let prefix = format!("pub_sub_rewrite_test_{}", unique_id());
        let source_topic = format!("{prefix}_y/a/z/b");
        let rewrite_topic = format!("{prefix}_y/z/b");

        let req = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_y/+/z/#"),
            dest_topic: format!("{prefix}_y/z/$2"),
            regex: format!("^{prefix}_y/(.+)/z/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req).await.unwrap();

        sleep(Duration::from_secs(2)).await;

        let network = "tcp";
        let protocol = 5;
        let qos = 1;
        let message = "pub_sub_rewrite_test mqtt message".to_string();

        // Test 1: Publish to source, receive on rewritten topic
        let sub_client_id = build_client_id(&format!("{prefix}_sub1"));
        let sub_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: sub_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let sub_cli = connect_server(&sub_properties);

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message
        };

        sub_cli.subscribe(&rewrite_topic, qos).unwrap();
        sleep(Duration::from_secs(1)).await;

        let pub_client_id = build_client_id(&format!("{prefix}_pub"));
        let pub_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: pub_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let pub_cli = connect_server(&pub_properties);

        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(source_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&pub_cli, msg, false);
        distinct_conn(pub_cli);

        subscribe_data_by_qos(&sub_cli, &rewrite_topic, qos, call_fn);
        distinct_conn(sub_cli);

        // Test 2: Subscribe to source, publish to rewritten topic
        let sub2_client_id = build_client_id(&format!("{prefix}_sub2"));
        let sub2_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: sub2_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let sub2_cli = connect_server(&sub2_properties);

        let pub2_client_id = build_client_id(&format!("{prefix}_pub2"));
        let pub2_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: pub2_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let pub2_cli = connect_server(&pub2_properties);

        let msg2 = MessageBuilder::new()
            .payload(message.clone())
            .topic(rewrite_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&pub2_cli, msg2, false);
        distinct_conn(pub2_cli);

        let call_fn2 = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message
        };
        subscribe_data_by_qos(&sub2_cli, &source_topic, qos, call_fn2);
        distinct_conn(sub2_cli);
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
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        cli.subscribe(&source_topic, qos).unwrap();
        sleep(Duration::from_secs(1)).await;
        cli.unsubscribe(&source_topic).unwrap();

        distinct_conn(cli);
    }

    /// EMQX documentation example: multiple rules with priority
    #[tokio::test]
    #[ignore = "needs integration environment"]
    async fn emqx_example_test() {
        let admin_client = create_test_env().await;
        let prefix = format!("emqx_example_{}", unique_id());

        let req1 = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_y/+/z/#"),
            dest_topic: format!("{prefix}_y/z/$2"),
            regex: format!("^{prefix}_y/(.+)/z/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req1).await.unwrap();
        sleep(Duration::from_millis(100)).await;

        let req2 = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_x/#"),
            dest_topic: format!("{prefix}_z/y/x/$1"),
            regex: format!("^{prefix}_x/y/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req2).await.unwrap();
        sleep(Duration::from_millis(100)).await;

        let req3 = CreateTopicRewriteReq {
            action: "All".to_string(),
            source_topic: format!("{prefix}_x/y/+"),
            dest_topic: format!("{prefix}_z/y/$1"),
            regex: format!("^{prefix}_x/y/(\\d+)$"),
        };
        admin_client.create_topic_rewrite(&req3).await.unwrap();
        sleep(Duration::from_secs(5)).await;

        let network = "tcp";
        let protocol = 5;
        let qos = 1;

        test_topic_rewrite(
            &prefix,
            &format!("{prefix}_y/a/z/b"),
            &format!("{prefix}_y/z/b"),
            network,
            protocol,
            qos,
        )
        .await;
        test_topic_rewrite(
            &prefix,
            &format!("{prefix}_y/def"),
            &format!("{prefix}_y/def"),
            network,
            protocol,
            qos,
        )
        .await;
        test_topic_rewrite(
            &prefix,
            &format!("{prefix}_x/1/2"),
            &format!("{prefix}_x/1/2"),
            network,
            protocol,
            qos,
        )
        .await;
        test_topic_rewrite(
            &prefix,
            &format!("{prefix}_x/y/2"),
            &format!("{prefix}_z/y/2"),
            network,
            protocol,
            qos,
        )
        .await;
        test_topic_rewrite(
            &prefix,
            &format!("{prefix}_x/y/z"),
            &format!("{prefix}_x/y/z"),
            network,
            protocol,
            qos,
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "needs integration environment"]
    async fn action_publish_only_test() {
        let admin_client = create_test_env().await;

        let prefix = format!("action_publish_{}", unique_id());
        let source_topic = format!("{prefix}_test/topic");
        let rewrite_topic = format!("{prefix}_rewritten/topic");

        let req = CreateTopicRewriteReq {
            action: "Publish".to_string(),
            source_topic: format!("{prefix}_test/#"),
            dest_topic: format!("{prefix}_rewritten/$1"),
            regex: format!("^{prefix}_test/(.+)$"),
        };
        admin_client.create_topic_rewrite(&req).await.unwrap();
        sleep(Duration::from_secs(5)).await;

        test_topic_rewrite(&prefix, &source_topic, &rewrite_topic, "tcp", 5, 1).await;
    }

    async fn test_topic_rewrite(
        prefix: &str,
        source_topic: &str,
        expected_topic: &str,
        network: &str,
        protocol: u32,
        qos: i32,
    ) {
        let message = format!("test message for {}", source_topic);

        let pub_client_id = build_client_id(&format!("{}_pub", prefix));
        let pub_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: pub_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let pub_cli = connect_server(&pub_properties);

        let sub_client_id = build_client_id(&format!("{}_sub", prefix));
        let sub_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: sub_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let sub_cli = connect_server(&sub_properties);

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message
        };
        sub_cli.subscribe(expected_topic, qos).unwrap();
        sleep(Duration::from_secs(1)).await;

        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(source_topic)
            .qos(qos)
            .finalize();
        publish_data(&pub_cli, msg, false);

        subscribe_data_by_qos(&sub_cli, expected_topic, qos, call_fn);

        distinct_conn(pub_cli);
        distinct_conn(sub_cli);
    }
}
