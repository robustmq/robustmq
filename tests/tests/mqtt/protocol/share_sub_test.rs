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
        let topic = format!("/tests/{}", unique_id());
        let group_name = unique_id();
        let sub_topic = format!("$share/{group_name}{topic}");
        single_test(topic, sub_topic, "share_single_subscribe_test").await;
    }

    #[tokio::test]
    async fn share_multi_subscribe_test() {
        let topic = format!("/multigroup/{}", unique_id());
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

        // publish
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
        // subscribe
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

        let r1_message_content = message_content.clone();
        let r1_flag = flag.to_string();
        let r1_sub_topic = sub_topic.clone();
        tokio::spawn(async move {
            // subscribe 1
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
                println!("subscribe 1:{}", payload);
                payload.contains(&r1_message_content)
            };

            subscribe_data_by_qos(&cli1, &r1_sub_topic, qos, call_fn);
            distinct_conn(cli1);
        });

        let r1_message_content = message_content.clone();
        let r1_flag = flag.to_string();
        let r1_sub_topic = sub_topic.clone();
        tokio::spawn(async move {
            // subscribe 1
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
                println!("subscribe 2:{}", payload);
                payload.contains(&r1_message_content)
            };

            subscribe_data_by_qos(&cli1, &r1_sub_topic, qos, call_fn);
            distinct_conn(cli1);
        });

        let r1_message_content = message_content.clone();
        let r1_flag = flag.to_string();
        let r1_sub_topic = sub_topic.clone();
        tokio::spawn(async move {
            // subscribe 1
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
                println!("subscribe 3:{}", payload);
                payload.contains(&r1_message_content)
            };

            subscribe_data_by_qos(&cli1, &r1_sub_topic, qos, call_fn);
            distinct_conn(cli1);
        });

        sleep(Duration::from_secs(1)).await;

        // publish
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
    }
}
