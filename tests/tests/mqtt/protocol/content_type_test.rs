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
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, publish_data,
            ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::uuid::unique_id;
    use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode};

    #[tokio::test]
    async fn content_type_test() {
        let network = "tcp";
        let qos = 1;
        let topic = format!(
            "/payload_format_indicator_connect_test/{}/{}/{}",
            unique_id(),
            network,
            qos
        );
        let client_id = build_client_id(format!("content_type_test_{network}_{qos}").as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        // publish
        let message_content = "content_type_test mqtt message".to_string();
        let mut props = Properties::new();
        let content_type = "lobo_json";
        props
            .push_string(PropertyCode::ContentType, content_type)
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
            let ct: String = msg
                .properties()
                .get_string(PropertyCode::ContentType)
                .unwrap();
            bl0 && ct == content_type
        };

        let res = subscribe_data_by_qos(&cli, &topic, qos, call_fn);
        assert!(res.is_ok());
        distinct_conn(cli);
    }
}
