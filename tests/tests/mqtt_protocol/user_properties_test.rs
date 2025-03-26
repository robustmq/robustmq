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
    use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode};

    use crate::mqtt_protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, network_types,
            publish_data, qos_list, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn user_properties_test() {
        for network in network_types() {
            for qos in qos_list() {
                let topic = format!("/tests/{}/{}/{}", unique_id(), network, qos);
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

                // publish
                let message_content = "mqtt message".to_string();
                let mut props = Properties::new();
                props
                    .push_u32(PropertyCode::MessageExpiryInterval, 50)
                    .unwrap();
                props
                    .push_string_pair(PropertyCode::UserProperty, "age", "1")
                    .unwrap();
                props
                    .push_string_pair(PropertyCode::UserProperty, "name", "robustmq")
                    .unwrap();

                let msg = MessageBuilder::new()
                    .properties(props.clone())
                    .payload(message_content.clone())
                    .topic(topic.clone())
                    .qos(qos)
                    .retained(false)
                    .finalize();
                publish_data(&cli, msg, false);

                // subscribe
                let call_fn = |msg: Message| {
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    let bl0 = payload == message_content;
                    let user_properties = msg
                        .properties()
                        .get_string_pair_at(PropertyCode::UserProperty, 0)
                        .unwrap();
                    let bl1 = user_properties.0 == *"age";
                    let bl2 = user_properties.1 == *"1";

                    let user_properties = msg
                        .properties()
                        .get_string_pair_at(PropertyCode::UserProperty, 1)
                        .unwrap();
                    let bl3 = user_properties.0 == *"name";
                    let bl4 = user_properties.1 == *"robustmq";

                    bl0 && bl1 && bl2 && bl3 && bl4
                };

                subscribe_data_by_qos(&cli, &topic, qos, call_fn);
                distinct_conn(cli);
            }
        }
    }
}
