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
    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, ssl_by_type,
            subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode, QOS_0};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn last_will_message_test() {
        let network = "tcp";
        let qos = QOS_0;
        let client_id = build_client_id(format!("last_will_message_test_{network}_{qos}").as_str());

        // create will message
        let mut props = Properties::new();
        props.push_u32(PropertyCode::WillDelayInterval, 10).unwrap();
        let content_type = "lobo_json";
        props
            .push_string(PropertyCode::ContentType, content_type)
            .unwrap();

        let will_message_content = "will message content".to_string();
        let will_topic = format!("/last_will_message_test/{}", unique_id());
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
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            will: Some(will),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        sleep(Duration::from_secs(3)).await;
        distinct_conn(cli);

        // subscribe
        let client_id =
            build_client_id(format!("last_will_message_test_sub_{network}_{qos}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            let bl0 = payload == will_message_content;
            let ct: String = msg
                .properties()
                .get_string(PropertyCode::ContentType)
                .unwrap();
            bl0 && ct == content_type
        };

        subscribe_data_by_qos(&cli, &will_topic, qos, call_fn);
        distinct_conn(cli);
    }
}
