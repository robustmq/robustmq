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
            publish_data, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, MessageBuilder, QOS_1};

    #[tokio::test]
    async fn client5_packet_size_test() {
        for network in network_types() {
            let topic = format!("/packet_tcp/{}/{}", unique_id(), network);

            let client_id =
                build_client_id(format!("client5_packet_size_test_{}", network).as_str());

            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(&network),
                ws: ws_by_type(&network),
                ssl: ssl_by_type(&network),
                ..Default::default()
            };
            let cli = connect_server(&client_properties);

            let packet_size = 128;

            let msg = MessageBuilder::new()
                .topic(topic.to_owned())
                .payload(
                    vec!['a'; (packet_size + 1) as usize]
                        .iter()
                        .collect::<String>(),
                )
                .qos(QOS_1)
                .finalize();

            publish_data(&cli, msg, true);

            let message2 = vec!['a'; packet_size as usize].iter().collect::<String>();
            let msg = MessageBuilder::new()
                .topic(topic.to_owned())
                .payload(message2.clone())
                .qos(QOS_1)
                .finalize();
            publish_data(&cli, msg, false);

            let call_fn = |msg: Message| {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                message2 == payload
            };

            subscribe_data_by_qos(&cli, &topic, 1, call_fn);
            distinct_conn(cli);
        }
    }
}
