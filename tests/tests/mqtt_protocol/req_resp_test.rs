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
    async fn client5_request_response_test() {
        for network in network_types() {
            for qos in qos_list() {
                let client_id = build_client_id(
                    format!("user_properties_test_{}_{}_{}", network, qos, unique_id()).as_str(),
                );

                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    request_response: true,
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                let request_topic = format!("/{}/request/{}/{}", unique_id(), network, qos);
                let response_topic = format!("/{}/response/{}/{}", unique_id(), network, qos);
                let correlation_data = "correlation_data_123456".to_string();

                // publish
                let mut props: Properties = Properties::new();
                props
                    .push_string(PropertyCode::ResponseTopic, response_topic.as_str())
                    .unwrap();

                props
                    .push_binary(
                        PropertyCode::CorrelationData,
                        serde_json::to_vec(&correlation_data).unwrap(),
                    )
                    .unwrap();

                let message_content = "message_content mqtt message".to_string();
                let msg = MessageBuilder::new()
                    .properties(props)
                    .topic(request_topic.clone())
                    .payload(message_content.clone())
                    .qos(qos)
                    .finalize();
                publish_data(&cli, msg, false);
                distinct_conn(cli);

                // subscribe request topic
                let client_id = build_client_id(
                    format!(
                        "user_properties_test_request_{}_{}_{}",
                        network,
                        qos,
                        unique_id()
                    )
                    .as_str(),
                );
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    request_response: true,
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);
                let call_fn = |msg: Message| {
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    let res = payload == message_content;

                    if res {
                        let topic = msg
                            .properties()
                            .get_string(PropertyCode::ResponseTopic)
                            .unwrap();
                        assert_eq!(topic, response_topic);

                        let c_data: Vec<u8> = msg
                            .properties()
                            .get_binary(PropertyCode::CorrelationData)
                            .unwrap();
                        let c_data_1 = serde_json::from_slice::<String>(&c_data).unwrap();

                        let client_id = build_client_id(
                            format!("user_properties_test_{}_{}_{}", network, qos, unique_id())
                                .as_str(),
                        );
                        let client_properties = ClientTestProperties {
                            mqtt_version: 5,
                            client_id: client_id.to_string(),
                            addr: broker_addr_by_type(&network),
                            ws: ws_by_type(&network),
                            ssl: ssl_by_type(&network),
                            request_response: true,
                            ..Default::default()
                        };
                        let cli = connect_server(&client_properties);

                        let msg = MessageBuilder::new()
                            .topic(topic)
                            .payload(c_data_1)
                            .qos(qos)
                            .finalize();

                        publish_data(&cli, msg, false);
                        distinct_conn(cli);
                    }
                    res
                };
                subscribe_data_by_qos(&cli, &request_topic, qos, call_fn);
                distinct_conn(cli);

                // subscribe request topic
                let client_id = build_client_id(
                    format!(
                        "user_properties_test_response_{}_{}_{}",
                        network,
                        qos,
                        unique_id()
                    )
                    .as_str(),
                );

                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    request_response: true,
                    ..Default::default()
                };

                let cli = connect_server(&client_properties);

                // publish
                // sub no_local

                let call_fn = |msg: Message| {
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    payload == correlation_data
                };

                subscribe_data_by_qos(&cli, &response_topic, qos, call_fn);
                distinct_conn(cli);
            }
        }
    }
}
