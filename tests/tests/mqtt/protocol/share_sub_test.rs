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
    use std::collections::HashSet;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, publish_data,
            ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, MessageBuilder};
    use tokio::time::sleep;

    #[tokio::test]
    async fn share_single_subscribe_test() {
        let topic = format!("/share_single_subscribe_test/{}", unique_id());
        let group_name = unique_id();
        let sub_topic = format!("$share/{group_name}{topic}");
        single_test(topic, sub_topic, "share_single_subscribe_test").await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn share_multi_subscribe_test() {
        let topic = format!("/share_multi_subscribe_test/{}", unique_id());
        let group_name = unique_id();
        let sub_topic = format!("$share/{group_name}{topic}");
        multi_test(
            topic.clone(),
            sub_topic.clone(),
            "share_multi_subscribe_test",
        )
        .await;
    }

    async fn single_test(pub_topic: String, sub_topic: String, flag: &str) {
        let network = "tcp";
        let qos = 1;
        let client_id = build_client_id(flag);
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let message_content = "share_subscribe_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .payload(message_content.clone())
            .topic(pub_topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        let client_id = build_client_id(flag);
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message_content
        };

        subscribe_data_by_qos(&cli, &sub_topic, qos, call_fn);
        distinct_conn(cli);
    }

    async fn multi_test(pub_topic: String, sub_topic: String, flag: &str) {
        let network = "tcp";
        let qos = 1;

        let (tx, rx) = channel::<(u32, String)>();

        let message_content = "share_subscribe_test mqtt message".to_string();

        let r1_message_content = message_content.clone();
        let r1_flag = format!("{}_sub1", flag);
        let r1_sub_topic = sub_topic.clone();
        let tx1 = tx.clone();
        tokio::spawn(async move {
            let client_id = build_client_id(&r1_flag);
            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(network),
                ws: ws_by_type(network),
                ssl: ssl_by_type(network),
                ..Default::default()
            };
            let cli1 = connect_server(&client_properties);
            let call_fn = |msg: Message| {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                tx1.send((1, payload.clone())).ok();
                payload.contains(&r1_message_content)
            };

            subscribe_data_by_qos(&cli1, &r1_sub_topic, qos, call_fn);
            distinct_conn(cli1);
        });

        let r2_message_content = message_content.clone();
        let r2_flag = format!("{}_sub2", flag);
        let r2_sub_topic = sub_topic.clone();
        let tx2 = tx.clone();
        tokio::spawn(async move {
            let client_id = build_client_id(&r2_flag);
            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(network),
                ws: ws_by_type(network),
                ssl: ssl_by_type(network),
                ..Default::default()
            };
            let cli2 = connect_server(&client_properties);
            let call_fn = |msg: Message| {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                tx2.send((2, payload.clone())).ok();
                payload.contains(&r2_message_content)
            };

            subscribe_data_by_qos(&cli2, &r2_sub_topic, qos, call_fn);
            distinct_conn(cli2);
        });

        let r3_message_content = message_content.clone();
        let r3_flag = format!("{}_sub3", flag);
        let r3_sub_topic = sub_topic.clone();
        let tx3 = tx.clone();
        tokio::spawn(async move {
            let client_id = build_client_id(&r3_flag);
            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(network),
                ws: ws_by_type(network),
                ssl: ssl_by_type(network),
                ..Default::default()
            };
            let cli3 = connect_server(&client_properties);
            let call_fn = |msg: Message| {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                tx3.send((3, payload.clone())).ok();
                payload.contains(&r3_message_content)
            };

            subscribe_data_by_qos(&cli3, &r3_sub_topic, qos, call_fn);
            distinct_conn(cli3);
        });

        sleep(Duration::from_secs(3)).await;

        let client_id = build_client_id(flag);
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        for i in 1..4 {
            let msg = MessageBuilder::new()
                .payload(format!("{},{}", message_content, i))
                .topic(pub_topic.clone())
                .qos(qos)
                .retained(false)
                .finalize();
            publish_data(&cli, msg, false);
        }
        distinct_conn(cli);

        sleep(Duration::from_secs(5)).await;
        drop(tx);

        let mut results = vec![];
        while let Ok(result) = rx.try_recv() {
            results.push(result);
        }

        assert_eq!(
            results.len(),
            3,
            "Expected 3 messages, but received {}",
            results.len()
        );

        let subscriber_ids: HashSet<_> = results.iter().map(|(id, _)| id).collect();
        assert_eq!(
            subscriber_ids.len(),
            3,
            "Each subscriber should receive one message, but only {} subscribers received messages",
            subscriber_ids.len()
        );

        let mut count_map = std::collections::HashMap::new();
        for (id, _) in &results {
            *count_map.entry(id).or_insert(0) += 1;
        }
        for (id, count) in count_map {
            assert_eq!(
                count, 1,
                "Subscriber {} should receive only 1 message, but received {}",
                id, count
            );
        }
    }
}
