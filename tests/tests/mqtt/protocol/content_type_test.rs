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

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, publish_data,
            ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };

    #[ignore]
    #[tokio::test]
    async fn payload_format_indicator_connect_test() {
        let network = "tcp";
        let qos = 1;
        let client_id =
            build_client_id(format!("payload_format_indicator_test_{network}_{qos}").as_str());

        // payload_format_indicator = 0 => success
        let will_message_content = "Hello, world".as_bytes();
        let will_topic = format!("/tests/{}", unique_id());
        let will = MessageBuilder::new()
            .payload(will_message_content)
            .topic(will_topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            will: Some(will),
            conn_is_err: false,
            ..Default::default()
        };
        let _ = connect_server(&client_properties);

        // payload_format_indicator = 0 => success
        let client_id =
            build_client_id(format!("payload_format_indicator_test_{network}_{qos}").as_str());

        let will_message_content = &[0xff, 0xfe, 0xfd];
        let will_topic = format!("/tests/{}", unique_id());
        let will = MessageBuilder::new()
            .payload(will_message_content)
            .topic(will_topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
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
        let will_topic = format!("/tests/{}", unique_id());
        let will = MessageBuilder::new()
            .properties(props.clone())
            .payload(will_message_content)
            .topic(will_topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            will: Some(will),
            conn_is_err: false,
            ..Default::default()
        };
        let _ = connect_server(&client_properties);

        // payload_format_indicator = 1 => success
        let client_id =
            build_client_id(format!("payload_format_indicator_test_{network}_{qos}").as_str());

        let will_message_content = &[0xff, 0xfe, 0xfd];
        let will_topic = format!("/tests/{}", unique_id());
        let will = MessageBuilder::new()
            .properties(props)
            .payload(will_message_content)
            .topic(will_topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
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
        let topic = format!("/tests/{}/{}/{}", unique_id(), network, qos);
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
            .retained(false)
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
            .retained(false)
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

    #[tokio::test]
    async fn content_type_test() {
        let network = "tcp";
        let qos = 1;
        let topic = format!("/tests/{}/{}/{}", unique_id(), network, qos);
        let client_id = build_client_id(format!("ucontent_type_test_{network}_{qos}").as_str());

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
            println!("content type:{ct}");
            bl0 && ct == content_type
        };

        subscribe_data_by_qos(&cli, &topic, qos, call_fn);
        distinct_conn(cli);
    }
}
