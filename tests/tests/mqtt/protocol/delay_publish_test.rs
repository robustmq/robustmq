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
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, publish_data,
            ssl_by_type, subscribe_data_with_options, uniq_topic, ws_by_type, SubscribeTestData,
        },
        ClientTestProperties,
    };
    use common_base::tools::now_second;
    use paho_mqtt::{Message, PropertyCode, SubscribeOptions, QOS_1};

    fn publish_delay_message(
        delay_topic: &str,
        message_content: &str,
        client_id_prefix: &str,
        network: &str,
    ) {
        let client_id = build_client_id(client_id_prefix);
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let msg = Message::new(delay_topic, message_content, QOS_1);
        publish_data(&cli, msg, false);
        distinct_conn(cli);
    }

    fn verify_delay_message(msg: &Message, expected_content: &str, expected_delay: u64) -> bool {
        let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
        if msg.properties().len() != 5 {
            println!("properties:{:?}", msg.properties());
            println!("payload:{payload}");
            return false;
        }

        if payload != expected_content {
            return false;
        }

        assert_eq!(msg.properties().len(), 5);
        let flag = msg
            .properties()
            .get_string_pair_at(PropertyCode::UserProperty, 0)
            .unwrap();

        let recv_ms = msg
            .properties()
            .get_string_pair_at(PropertyCode::UserProperty, 1)
            .unwrap();

        let target_ms = msg
            .properties()
            .get_string_pair_at(PropertyCode::UserProperty, 2)
            .unwrap();

        let save_ms = msg
            .properties()
            .get_string_pair_at(PropertyCode::UserProperty, 3)
            .unwrap();

        assert_eq!(flag.0, *"delay_message_flag");
        assert_eq!(flag.1, *"true");

        let recv_ms1 = recv_ms.1.parse::<i64>().unwrap();
        let target_ms1 = target_ms.1.parse::<i64>().unwrap();
        let save_ms1 = save_ms.1.parse::<i64>().unwrap();

        let drift = save_ms1 - target_ms1;
        println!(
            "expected_delay:{},now:{},recv_ms1:{},target_ms1:{},save_ms1:{},drift:{}s",
            expected_delay,
            now_second(),
            recv_ms1,
            target_ms1,
            save_ms1,
            drift
        );
        save_ms1 >= target_ms1 && drift <= 2
    }

    async fn test_delay_publish(
        delay_topic: &str,
        sub_topic: &str,
        message_content: &str,
        expected_delay: u64,
        test_name: &str,
    ) {
        let network = "tcp";
        let qos = 1;

        publish_delay_message(delay_topic, message_content, test_name, network);

        let client_id = build_client_id(&format!("{}_sub", test_name));
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let call_fn =
            move |msg: Message| verify_delay_message(&msg, message_content, expected_delay);

        let subscribe_test_data = SubscribeTestData {
            sub_topic: sub_topic.to_string(),
            sub_qos: qos,
            subscribe_options: SubscribeOptions::default(),
            subscribe_properties: None,
        };

        let res = subscribe_data_with_options(&cli, subscribe_test_data, call_fn).await;
        assert!(res.is_ok(), "subscribe_data_with_options failed: {:?}", res);
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn delay_publish_test() {
        let network = "tcp";
        let qos = 1;

        for t in [10, 20, 30] {
            let uniq_tp = uniq_topic();
            let topic = format!("$delayed/{}/{}", t, &uniq_tp[1..]);

            // publish
            let client_id = build_client_id(format!("delay_publish_test_{network}_{qos}").as_str());
            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(network),
                ..Default::default()
            };
            let cli = connect_server(&client_properties);

            let message_content = format!("delay_publish_test mqtt message,{uniq_tp},{t}");
            let msg = Message::new(topic.clone(), message_content.clone(), QOS_1);
            publish_data(&cli, msg, false);
            distinct_conn(cli);

            // subscribe +
            let client_id = build_client_id(format!("delay_publish_test_{network}_{qos}").as_str());

            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(network),
                ws: ws_by_type(network),
                ssl: ssl_by_type(network),
                ..Default::default()
            };
            let cli = connect_server(&client_properties);

            let sub_topic = uniq_tp;

            let call_fn = |msg: Message| {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                // Avoid auto-subscribe to subscribe to data
                if msg.properties().len() != 5 {
                    println!("properties:{:?}", msg.properties());
                    println!("payload:{payload}");
                    return false;
                }

                if payload != message_content {
                    return false;
                }

                assert_eq!(msg.properties().len(), 5);
                let flag = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 0)
                    .unwrap();

                let recv_ms = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 1)
                    .unwrap();

                let target_ms = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 2)
                    .unwrap();

                let save_ms = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 3)
                    .unwrap();

                assert_eq!(flag.0, *"delay_message_flag");
                assert_eq!(flag.1, *"true");

                let recv_ms1 = recv_ms.1.parse::<i64>().unwrap();
                let target_ms1 = target_ms.1.parse::<i64>().unwrap();
                let save_ms1 = save_ms.1.parse::<i64>().unwrap();

                let drift = save_ms1 - target_ms1;
                println!(
                    "t:{},now:{},recv_ms1:{},target_ms1:{},save_ms1:{},drift:{}s",
                    t,
                    now_second(),
                    recv_ms1,
                    target_ms1,
                    save_ms1,
                    drift
                );
                save_ms1 >= target_ms1 && drift <= 2
            };

            let subscribe_test_data = SubscribeTestData {
                sub_topic: sub_topic.clone(),
                sub_qos: qos,
                subscribe_options: SubscribeOptions::default(),
                subscribe_properties: None,
            };

            let res = subscribe_data_with_options(&cli, subscribe_test_data, call_fn).await;
            assert!(res.is_ok(), "subscribe_data_with_options failed: {:?}", res);
            distinct_conn(cli);
        }
    }

    #[tokio::test]
    async fn delay_publish_15s_test() {
        let uniq_tp = uniq_topic();
        let message_content = format!("delay 15s test,{uniq_tp}");
        test_delay_publish(
            "$delayed/15/x/y",
            "/x/y",
            &message_content,
            15,
            "delay_publish_15s_test",
        )
        .await;
    }

    #[tokio::test]
    async fn delay_publish_30s_test() {
        let uniq_tp = uniq_topic();
        let message_content = format!("delay 30s test,{uniq_tp}");
        test_delay_publish(
            "$delayed/30/a/b",
            "/a/b",
            &message_content,
            30,
            "delay_publish_30s_test",
        )
        .await;
    }

    #[tokio::test]
    async fn delay_publish_system_topic_test() {
        let uniq_tp = uniq_topic();
        let message_content = format!("system topic delay test,{uniq_tp}");
        test_delay_publish(
            "$delayed/15/$SYS/topic",
            "/$SYS/topic",
            &message_content,
            15,
            "delay_publish_sys_test",
        )
        .await;
    }
}
