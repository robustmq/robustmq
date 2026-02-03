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
            broker_addr_by_type, build_client_id, connect_server, publish_data, ssl_by_type,
            ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::uuid::unique_id;
    use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode};

    /// Build properties with payload format indicator
    fn build_payload_format_properties(format_indicator: u8) -> Properties {
        let mut props = Properties::new();
        props
            .push_int(
                PropertyCode::PayloadFormatIndicator,
                format_indicator as i32,
            )
            .unwrap();
        props
    }

    /// Build a will message for testing
    fn build_will_message(payload: &[u8], qos: i32, format_indicator: Option<u8>) -> Message {
        let topic = format!("/payload_format_indicator_connect_test/{}", unique_id());
        let mut builder = MessageBuilder::new().payload(payload).topic(topic).qos(qos);

        if let Some(indicator) = format_indicator {
            builder = builder.properties(build_payload_format_properties(indicator));
        }

        builder.finalize()
    }

    /// Test connection with specified will message and expected result
    fn test_connect_with_will(network: &str, will: Message, should_succeed: bool) {
        let client_id = build_client_id(&format!(
            "payload_format_indicator_test_{}_{}",
            network,
            unique_id()
        ));

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id,
            addr: broker_addr_by_type(network),
            will: Some(will),
            conn_is_err: !should_succeed,
            ..Default::default()
        };

        let _ = connect_server(&client_properties);
    }

    #[tokio::test]
    async fn payload_format_indicator_connect_test() {
        let network = "tcp";
        let qos = 1;

        // Test 1: payload_format_indicator = 0 (default), payload = UTF-8 text => success
        let will = build_will_message("Hello, world".as_bytes(), qos, None);
        test_connect_with_will(network, will, true);

        // Test 2: payload_format_indicator = 0 (default), payload = binary => success
        let will = build_will_message(&[0xff, 0xfe, 0xfd], qos, None);
        test_connect_with_will(network, will, true);

        // Test 3: payload_format_indicator = 1 (UTF-8), payload = valid UTF-8 => success
        let will = build_will_message("Hello, world".as_bytes(), qos, Some(1));
        test_connect_with_will(network, will, true);

        // Test 4: payload_format_indicator = 1 (UTF-8), payload = invalid UTF-8 => failure
        let will = build_will_message(&[0xff, 0xfe, 0xfd], qos, Some(1));
        test_connect_with_will(network, will, false);
    }

    #[tokio::test]
    async fn payload_format_indicator_publish_test() {
        let network = "tcp";
        let qos = 1;
        let topic = format!(
            "/payload_format_indicator_publish_test/{}/{}/{}",
            unique_id(),
            network,
            qos
        );

        // Connect client for testing
        let client_id = build_client_id(&format!(
            "payload_format_indicator_publish_test_{}_{}",
            network,
            unique_id()
        ));
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id,
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        // Test 1: payload_format_indicator = 0 (default), payload = UTF-8 text => success
        let msg = MessageBuilder::new()
            .payload("Hello, world".as_bytes())
            .topic(topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        publish_data(&cli, msg, false);

        // Test 2: payload_format_indicator = 0 (default), payload = binary => success
        let msg = MessageBuilder::new()
            .payload([0xff, 0xfe, 0xfd])
            .topic(topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        // Test 3: payload_format_indicator = 1 (UTF-8), payload = valid UTF-8 => success
        let msg = MessageBuilder::new()
            .properties(build_payload_format_properties(1))
            .payload("Hello, world".as_bytes())
            .topic(topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        // Test 4: payload_format_indicator = 1 (UTF-8), payload = invalid UTF-8 => failure
        let msg = MessageBuilder::new()
            .properties(build_payload_format_properties(1))
            .payload([0xff, 0xfe, 0xfd])
            .topic(topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        publish_data(&cli, msg, true);
    }
}
