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
    use crate::mqtt_protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, network_types,
            publish_data, qos_list, ssl_by_type, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, RetainHandling, SubscribeOptions, QOS_1};

    #[tokio::test]
    async fn sub_exclusive_test() {
        for network in network_types() {
            for qos in qos_list() {
                let topic = format!("/tests/{}", unique_id());
                let sub_exclusive_topics: &[String; 1] = &[format!("$exclusive{}", topic.clone())];
                let sub_opts = &[SubscribeOptions::new(
                    true,
                    false,
                    RetainHandling::DontSendRetained,
                )];

                // publish
                let client_id =
                    build_client_id(format!("user_properties_test_{}_{}", network, qos).as_str());

                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                let message_content = "mqtt message".to_string();
                let msg = Message::new(topic.clone(), message_content.clone(), QOS_1);
                publish_data(&cli, msg, false);
                distinct_conn(cli);

                // subscribe exclusive topic
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli1 = connect_server(&client_properties);
                let result =
                    cli1.subscribe_many_with_options(sub_exclusive_topics, &[qos], sub_opts, None);
                assert!(result.is_ok());

                // subscribe topic success
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli2 = connect_server(&client_properties);
                let result = cli2.subscribe_many_with_options(&[topic], &[qos], sub_opts, None);
                println!("{:?}", result);
                assert!(result.is_ok());

                // subscribe exclusive topic fail
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli3 = connect_server(&client_properties);
                let result =
                    cli3.subscribe_many_with_options(sub_exclusive_topics, &[qos], sub_opts, None);
                assert!(result.is_err());
                distinct_conn(cli1);
                distinct_conn(cli2);
                distinct_conn(cli3);
            }
        }
    }
}
