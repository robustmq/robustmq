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
    use mqtt_broker::handler::constant::{
        SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE,
    };
    use paho_mqtt::{Message, MessageBuilder, PropertyCode};

    use crate::mqtt_protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, network_types,
            publish_data, qos_list, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn retain_message_sub_test() {
        for network in network_types() {
            for qos in qos_list() {
                let topic = format!("/tests/{}/{}/{}", unique_id(), network, qos);
                let client_id = build_client_id(
                    format!("retain_message_sub_test_{}_{}", network, qos).as_str(),
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

                // publish
                let message = "mqtt message".to_string();
                let msg = MessageBuilder::new()
                    .payload(message.clone())
                    .topic(topic.clone())
                    .qos(qos)
                    .retained(true)
                    .finalize();
                publish_data(&cli, msg, false);

                // subscribe
                let call_fn = |msg: Message| {
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    if payload == message {
                        if let Some(raw) = msg
                            .properties()
                            .get_string_pair_at(PropertyCode::UserProperty, 0)
                        {
                            if raw.0 == *SUB_RETAIN_MESSAGE_PUSH_FLAG
                                && raw.1 == *SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE
                            {
                                return true;
                            }
                        }
                    }
                    false
                };

                subscribe_data_by_qos(&cli, &topic, qos, call_fn);
                distinct_conn(cli);
            }
        }
    }
}
