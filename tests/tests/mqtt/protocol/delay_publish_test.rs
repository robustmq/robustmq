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
    use common_base::tools::now_second;
    use paho_mqtt::{Message, PropertyCode, SubscribeOptions, QOS_1};

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, publish_data,
            ssl_by_type, subscribe_data_with_options, uniq_topic, ws_by_type, SubscribeTestData,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn delay_publish_test() {
        let network = "tcp";
        let qos = 1;

        for t in [2, 4, 6] {
            let uniq_tp = uniq_topic();
            let topic = format!("$delayed/{t}{uniq_tp}");

            // publish
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

            let message_content = format!("delay_publish_test mqtt message,{uniq_tp}");
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
                if msg.properties().len() != 4 {
                    println!("properties:{:?}", msg.properties());
                    println!("payload:{payload}");
                    return false;
                }

                assert_eq!(msg.properties().len(), 4);
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

                assert_eq!(flag.0, *"delay_message_flag");
                assert_eq!(flag.1, *"true");

                let recv_ms1 = recv_ms.1.parse::<i64>().unwrap();
                let target_ms2 = target_ms.1.parse::<i64>().unwrap();
                let diff = target_ms2 - recv_ms1;
                assert_eq!(diff, t as i64);
                println!(
                    "t:{},now:{},target_ms2:{},diff:{}",
                    t,
                    now_second(),
                    target_ms2,
                    now_second() - target_ms2 as u64
                );
                assert!((now_second() - target_ms2 as u64) < 3);
                payload == message_content
            };

            let subscribe_test_data = SubscribeTestData {
                sub_topic: sub_topic.clone(),
                sub_qos: qos,
                subscribe_options: SubscribeOptions::default(),
                subscribe_properties: None,
            };

            subscribe_data_with_options(&cli, subscribe_test_data, call_fn);
            distinct_conn(cli);
        }
    }
}
