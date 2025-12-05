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

    use common_base::tools::unique_id;
    use paho_mqtt::{Message, SubscribeOptions, QOS_1};

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, network_types,
            publish_data, qos_list, ssl_by_type, subscribe_data_with_options, ws_by_type,
            SubscribeTestData,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn sub_wildcards_test() {
        for network in network_types() {
            for qos in qos_list() {
                let uniq = unique_id();
                let topic = format!("/tests/v1/v2/{uniq}");

                // publish
                let client_id =
                    build_client_id(format!("sub_wildcards_test_{network}_{qos}").as_str());

                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                let message_content = "sub_wildcards_test mqtt message".to_string();
                let msg = Message::new(topic.clone(), message_content.clone(), QOS_1);
                publish_data(&cli, msg, false);
                distinct_conn(cli);

                // subscribe with + wildcard
                let client_id: String =
                    build_client_id(format!("sub_wildcards_test_+_{network}_{qos}").as_str());

                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                let sub_topic = format!("/tests/v1/+/{uniq}");
                let call_fn = |msg: Message| {
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
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

                // subscribe with # wildcard (multi-level)
                let client_id =
                    build_client_id(format!("sub_wildcards_test_#_{network}_{qos}").as_str());

                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                // Fix: /tests/# matches /tests/v1/v2/{uniq}
                let sub_topic = "/tests/#".to_string();
                let call_fn = |msg: Message| {
                    let payload = match String::from_utf8(msg.payload().to_vec()) {
                        Ok(payload) => payload,
                        Err(_) => return false,
                    };
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
}
