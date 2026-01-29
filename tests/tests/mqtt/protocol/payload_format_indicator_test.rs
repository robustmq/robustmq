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

    #[tokio::test]
    async fn payload_format_indicator_connect_test() {
        let network = "tcp";
        let qos = 1;
        let client_id =
            build_client_id(format!("payload_format_indicator_test_{network}_{qos}").as_str());

        // payload_format_indicator = 0 => success
        let will_message_content = "Hello, world".as_bytes();
        let will_topic = format!("/payload_format_indicator_connect_test/{}", unique_id());
        let will = MessageBuilder::new()
            .payload(will_message_content)
            .topic(will_topic.clone())
            .qos(qos)
            .finalize();

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            will: Some(will),
            conn_is_err: false,
            ..Default::default()
        };
        let _ = connect_server(&client_properties);

        // payload_format_indicator = 0 => success
        let client_id =
            build_client_id(format!("payload_format_indicator_test_{network}_{qos}").as_str());

        let will_message_content = &[0xff, 0xfe, 0xfd];
        let will_topic = format!("/payload_format_indicator_connect_test/{}", unique_id());
        let will = MessageBuilder::new()
            .payload(will_message_content)
            .topic(will_topic.clone())
            .qos(qos)
            .finalize();
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            will: Some(will),
            conn_is_err: false,
            ..Default::default()
        };
        let _ = connect_server(&client_properties);

        // payload_format_indicator = 1 => success
        let mut props = Properties::new();
        props
            .push_int(PropertyCode::PayloadFormatIndicator, 1)
            .unwrap();
        let will_message_content = "Hello, world".as_bytes();
        let will_topic = format!("/payload_format_indicator_connect_test/{}", unique_id());
        let will = MessageBuilder::new()
            .properties(props.clone())
            .payload(will_message_content)
            .topic(will_topic.clone())
            .qos(qos)
            .finalize();
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            will: Some(will),
            conn_is_err: false,
            ..Default::default()
        };
        let _ = connect_server(&client_properties);

        // payload_format_indicator = 1 => false
        let client_id =
            build_client_id(format!("payload_format_indicator_test_{network}_{qos}").as_str());

        let will_message_content = &[0xff, 0xfe, 0xfd];
        let will_topic = format!("/payload_format_indicator_connect_test/{}", unique_id());
        let will = MessageBuilder::new()
            .properties(props)
            .payload(will_message_content)
            .topic(will_topic.clone())
            .qos(qos)
            .finalize();
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            will: Some(will),
            conn_is_err: true,
            ..Default::default()
        };
        let _ = connect_server(&client_properties);
    }

    #[tokio::test]
    async fn payload_format_indicator_publish_test() {
        let network = "tcp";
        let qos = 1;
        let topic = format!(
            "/payload_format_indicator_connect_test/{}/{}/{}",
            unique_id(),
            network,
            qos
        );
        let client_id = build_client_id(
            format!("payload_format_indicator_publish_test_{network}_{qos}").as_str(),
        );
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };

        // payload_format_indicator = 0 => success
        let cli = connect_server(&client_properties);
        let message_content = "Hello, world".as_bytes();
        let msg = MessageBuilder::new()
            .payload(message_content)
            .topic(topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        publish_data(&cli, msg, false);

        // payload_format_indicator = 0 => success
        let message_content = &[0xff, 0xfe, 0xfd];
        let msg = MessageBuilder::new()
            .payload(message_content)
            .topic(topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        let mut props = Properties::new();
        props
            .push_int(PropertyCode::PayloadFormatIndicator, 1)
            .unwrap();

        // payload_format_indicator = 0 => success
        let cli = connect_server(&client_properties);
        let message_content = "Hello, world".as_bytes();
        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(message_content)
            .topic(topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        // payload_format_indicator = 0 => success
        let message_content = &[0xff, 0xfe, 0xfd];
        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(message_content)
            .topic(topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        publish_data(&cli, msg, true);
    }
}
