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
    use paho_mqtt::{MessageBuilder, Properties, PropertyCode, SubscribeOptions};

    #[tokio::test]
    async fn sub_identifier_test() {
        for network in network_types() {
            for qos in qos_list() {
                let client_id =
                    build_client_id(format!("user_properties_test_{}_{}", network, qos).as_str());

                let topic = unique_id();
                let topic1 = format!("/test_tcp/{}/+", topic);
                let topic2 = format!("/test_tcp/{}/test", topic);
                let topic3 = format!("/test_tcp/{}/test_one", topic);

                // publish
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };

                let cli = connect_server(&client_properties);
                let message_content = format!("mqtt  message");
                let msg = MessageBuilder::new()
                    .topic(topic2.clone())
                    .payload(message_content.clone())
                    .qos(qos)
                    .finalize();
                publish_data(&cli, msg, false);
                distinct_conn(cli);

                // subscribe
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

                // sub1
                let cli = connect_server(&client_properties);
                let mut props: Properties = Properties::new();
                props
                    .push_int(PropertyCode::SubscriptionIdentifier, 1)
                    .unwrap();
                let res = cli.subscribe_many_with_options(
                    &[topic1.clone()],
                    &[qos],
                    &[SubscribeOptions::default()],
                    Some(props),
                );
                assert!(res.is_ok());

                // sub2
                let mut props: Properties = Properties::new();
                props
                    .push_int(PropertyCode::SubscriptionIdentifier, 2)
                    .unwrap();

                let res = cli.subscribe_many_with_options(
                    &[topic2.clone()],
                    &[qos],
                    &[SubscribeOptions::default()],
                    Some(props),
                );
                assert!(res.is_ok());

                // sub data
                let mut r_one = false;
                let mut r_two = false;
                let rx = cli.start_consuming();

                for message in rx.iter() {
                    if let Some(msg) = message {
                        let sub_identifier = msg
                            .properties()
                            .get_int(PropertyCode::SubscriptionIdentifier)
                            .unwrap();

                        println!("{:?} sub_identifier: {}", msg, sub_identifier);

                        match sub_identifier {
                            1 => {
                                r_one = true;
                            }
                            2 => {
                                r_two = true;
                            }
                            _ => {
                                panic!("sub_identifier error");
                            }
                        }
                    }
                    if r_one && r_two {
                        break;
                    }
                }

                // publish data
                let msg = MessageBuilder::new()
                    .topic(topic3.clone())
                    .payload(message_content.clone())
                    .qos(qos)
                    .finalize();
                publish_data(&cli, msg, false);

                if let Some(msg) = rx.iter().flatten().next() {
                    let sub_identifier = msg
                        .properties()
                        .get_int(PropertyCode::SubscriptionIdentifier)
                        .unwrap();

                    assert_eq!(sub_identifier, 1);

                    println!("{msg:?}");
                    println!("{sub_identifier:?}");
                }

                distinct_conn(cli);
            }
        }
    }
}
