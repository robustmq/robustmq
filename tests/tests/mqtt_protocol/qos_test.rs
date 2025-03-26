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

use super::common::build_client_id;

#[cfg(test)]
mod tests {
    use super::*;

    use common_base::tools::unique_id;
    use paho_mqtt::{Message, MessageBuilder};

    use crate::mqtt_protocol::{
        common::{
            broker_addr_by_type, connect_server, distinct_conn, network_types, protocol_versions,
            publish_data, qos_list, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn publish_qos_test() {
        for protocol in protocol_versions() {
            for network in network_types() {
                for qos in qos_list() {
                    let topic = format!("/tests/{}/{}/{}", unique_id(), network, qos);
                    let client_id =
                        build_client_id(format!("publish_qos_test_{}_{}", network, qos).as_str());

                    let client_properties = ClientTestProperties {
                        mqtt_version: protocol,
                        client_id: client_id.to_string(),
                        addr: broker_addr_by_type(&network),
                        ws: ws_by_type(&network),
                        ssl: ssl_by_type(&network),
                        ..Default::default()
                    };
                    let cli = connect_server(&client_properties);

                    // publish retain
                    let message = "mqtt message".to_string();
                    let msg = MessageBuilder::new()
                        .payload(message.clone())
                        .topic(topic.clone())
                        .qos(qos)
                        .retained(true)
                        .finalize();
                    publish_data(&cli, msg, false);

                    // publish no retain
                    let message = "mqtt message".to_string();
                    let msg = MessageBuilder::new()
                        .payload(message.clone())
                        .topic(topic.clone())
                        .qos(qos)
                        .retained(false)
                        .finalize();
                    publish_data(&cli, msg, false);
                    distinct_conn(cli);
                }
            }
        }
    }

    #[tokio::test]
    async fn subscribe_qos_test() {
        for protocol in protocol_versions() {
            for network in network_types() {
                for qos in qos_list() {
                    let topic = format!("/tests/{}/{}/{}", unique_id(), network, qos);
                    let client_id =
                        build_client_id(format!("subscribe_qos_test_{}_{}", network, qos).as_str());
                    let client_properties = ClientTestProperties {
                        mqtt_version: protocol,
                        client_id: client_id.to_string(),
                        addr: broker_addr_by_type(&network),
                        ws: ws_by_type(&network),
                        ssl: ssl_by_type(&network),
                        ..Default::default()
                    };
                    let cli = connect_server(&client_properties);

                    // publish retain
                    let message = "mqtt message".to_string();
                    let msg = MessageBuilder::new()
                        .payload(message.clone())
                        .topic(topic.clone())
                        .qos(qos)
                        .finalize();
                    publish_data(&cli, msg, false);

                    // subscribe
                    let call_fn = |msg: Message| {
                        let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                        if payload == message {
                            return true;
                        }
                        false
                    };
                    subscribe_data_by_qos(&cli, &topic, qos, call_fn);
                    distinct_conn(cli);
                }
            }
        }
    }
}
