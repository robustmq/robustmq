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

    use common_base::tools::unique_id;
    use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode};
    use tokio::time::sleep;

    use crate::mqtt_protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, network_types,
            qos_list, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn last_will_message_test() {
        for network in network_types() {
            for qos in qos_list() {
                let client_id =
                    build_client_id(format!("last_will_message_test_{}_{}", network, qos).as_str());

                // create will message
                let mut props = Properties::new();
                props.push_u32(PropertyCode::WillDelayInterval, 2).unwrap();
                let will_message_content = "will message content".to_string();
                let will_topic = format!("/tests/{}", unique_id());
                let will = MessageBuilder::new()
                    .properties(props)
                    .payload(will_message_content.clone())
                    .topic(will_topic.clone())
                    .qos(qos)
                    .retained(false)
                    .finalize();

                // create connection
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    will: Some(will),
                    ..Default::default()
                };
                let _ = connect_server(&client_properties);
                sleep(Duration::from_secs(3)).await;

                // subscribe
                let client_id = build_client_id(
                    format!("last_will_message_test_sub_{}_{}", network, qos).as_str(),
                );
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                let call_fn = |msg: Message| {
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    println!("retain message:{}", payload.clone());
                    payload == will_message_content
                };

                subscribe_data_by_qos(&cli, &will_topic, qos, call_fn);
                distinct_conn(cli);
            }
        }
    }
}
