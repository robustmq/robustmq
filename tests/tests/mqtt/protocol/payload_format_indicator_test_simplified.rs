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
    use common_base::tools::unique_id;
    use paho_mqtt::{MessageBuilder, Properties, PropertyCode};

    /// Test data structure for payload format indicator tests
    struct PayloadTestCase {
        name: &'static str,
        format_indicator: Option<u8>, // None = not set (default 0)
        payload: Vec<u8>,
        should_succeed: bool,
    }

    impl PayloadTestCase {
        fn build_properties(&self) -> Option<Properties> {
            self.format_indicator.map(|indicator| {
                let mut props = Properties::new();
                props
                    .push_int(PropertyCode::PayloadFormatIndicator, indicator as i32)
                    .unwrap();
                props
            })
        }

        fn build_will_message(&self, qos: i32) -> paho_mqtt::Message {
            let topic = format!("/test/payload_format_indicator/{}", unique_id());
            let mut builder = MessageBuilder::new()
                .payload(self.payload.clone())
                .topic(topic)
                .qos(qos);

            if let Some(props) = self.build_properties() {
                builder = builder.properties(props);
            }

            builder.finalize()
        }

        fn build_publish_message(&self, topic: &str, qos: i32) -> paho_mqtt::Message {
            let mut builder = MessageBuilder::new()
                .payload(self.payload.clone())
                .topic(topic)
                .qos(qos)
                .retained(false);

            if let Some(props) = self.build_properties() {
                builder = builder.properties(props);
            }

            builder.finalize()
        }
    }

    fn get_test_cases() -> Vec<PayloadTestCase> {
        vec![
            PayloadTestCase {
                name: "format=0 (default), payload=UTF-8 text",
                format_indicator: None,
                payload: "Hello, world".as_bytes().to_vec(),
                should_succeed: true,
            },
            PayloadTestCase {
                name: "format=0 (default), payload=binary",
                format_indicator: None,
                payload: vec![0xff, 0xfe, 0xfd],
                should_succeed: true,
            },
            PayloadTestCase {
                name: "format=1 (UTF-8), payload=valid UTF-8",
                format_indicator: Some(1),
                payload: "Hello, world".as_bytes().to_vec(),
                should_succeed: true,
            },
            PayloadTestCase {
                name: "format=1 (UTF-8), payload=invalid UTF-8",
                format_indicator: Some(1),
                payload: vec![0xff, 0xfe, 0xfd],
                should_succeed: false,
            },
        ]
    }

    #[tokio::test]
    async fn payload_format_indicator_connect_test() {
        let network = "tcp";
        let qos = 1;

        for test_case in get_test_cases() {
            println!("Testing CONNECT: {}", test_case.name);

            let client_id = build_client_id(&format!(
                "payload_format_connect_{}_{}",
                network,
                unique_id()
            ));

            let will = test_case.build_will_message(qos);

            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id,
                addr: broker_addr_by_type(network),
                will: Some(will),
                conn_is_err: !test_case.should_succeed,
                ..Default::default()
            };

            let _ = connect_server(&client_properties);
        }
    }

    #[tokio::test]
    async fn payload_format_indicator_publish_test() {
        let network = "tcp";
        let qos = 1;
        let topic = format!(
            "/test/payload_format_publish/{}/{}/{}",
            unique_id(),
            network,
            qos
        );

        // Connect once for all tests
        let client_id = build_client_id(&format!(
            "payload_format_publish_{}_{}",
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

        for test_case in get_test_cases() {
            println!("Testing PUBLISH: {}", test_case.name);

            let msg = test_case.build_publish_message(&topic, qos);
            publish_data(&cli, msg, !test_case.should_succeed);
        }
    }

    #[tokio::test]
    async fn payload_format_indicator_extended_test() {
        let network = "tcp";

        // Test with different QoS levels
        for qos in [0, 1, 2] {
            println!("Testing with QoS {}", qos);

            let client_id = build_client_id(&format!(
                "payload_format_qos_{}_{}_{}",
                qos,
                network,
                unique_id()
            ));

            // Test case: format=1 with invalid UTF-8 should fail
            let mut props = Properties::new();
            props
                .push_int(PropertyCode::PayloadFormatIndicator, 1)
                .unwrap();

            let will = MessageBuilder::new()
                .properties(props)
                .payload(vec![0xff, 0xfe, 0xfd])
                .topic(format!("/test/qos/{}", unique_id()))
                .qos(qos)
                .finalize();

            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id,
                addr: broker_addr_by_type(network),
                will: Some(will),
                conn_is_err: true,
                ..Default::default()
            };

            let _ = connect_server(&client_properties);
        }
    }
}
