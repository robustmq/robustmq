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
            publish_data, qos_list, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode, QOS_1};

    #[tokio::test]
    async fn topic_alias_test() {
        for network in network_types() {
            for qos in qos_list() {
                let topic = format!("/tests/{}/{}/{}", unique_id(), network, qos);
                let client_id =
                    build_client_id(format!("topic_alias_test_{}_{}", network, qos).as_str());

                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                // publish first data
                let topic_alias = 1;
                let message_content = "mqtt message".to_string();
                let mut props = Properties::new();
                props
                    .push_u16(PropertyCode::TopicAlias, topic_alias)
                    .unwrap();

                let msg = MessageBuilder::new()
                    .properties(props.clone())
                    .payload(message_content.clone())
                    .topic(topic.clone())
                    .qos(qos)
                    .retained(false)
                    .finalize();
                publish_data(&cli, msg, false);

                // publish 1 success
                let message_content2 = "mqtt message alias".to_string();
                let msg = MessageBuilder::new()
                    .properties(props.clone())
                    .payload(message_content2.clone())
                    .qos(QOS_1)
                    .retained(false)
                    .finalize();
                publish_data(&cli, msg, false);

                // publisb 2 fail
                props.push_u16(PropertyCode::TopicAlias, 2).unwrap();
                let msg = MessageBuilder::new()
                    .properties(props.clone())
                    .payload(message_content2.clone())
                    .qos(QOS_1)
                    .retained(false)
                    .finalize();
                publish_data(&cli, msg, true);

                // subscribe
                let call_fn = |msg: Message| {
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    payload == message_content2
                };

                subscribe_data_by_qos(&cli, &topic, qos, call_fn);
                distinct_conn(cli);
            }
        }
    }
}
