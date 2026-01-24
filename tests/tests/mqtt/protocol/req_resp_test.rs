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
            broker_addr_by_type, build_client_id, build_conn_pros, build_create_conn_pros,
            connect_server, distinct_conn, password, publish_data, ssl_by_type,
            subscribe_data_by_qos, username, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::tools::unique_id;
    use mqtt_broker::core::connection::REQUEST_RESPONSE_PREFIX_NAME;
    use paho_mqtt::{
        Client, ConnectOptionsBuilder, CreateOptionsBuilder, Message, MessageBuilder, Properties,
        PropertyCode,
    };

    #[tokio::test]
    async fn connect_request_response_information_test() {
        let addr = broker_addr_by_type("tcp");

        // RequestResponseInformation = 1
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr.clone())
            .client_id(unique_id())
            .finalize();
        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::new_v5();
        let mut props = Properties::new();
        props
            .push_val(PropertyCode::RequestResponseInformation, 1)
            .unwrap();

        conn_opts
            .clean_session(false)
            .properties(props)
            .user_name(username())
            .password(password());
        let conn_opts = conn_opts.finalize();
        let resp = cli.connect(conn_opts).unwrap();
        let properties = resp.properties();
        let data = properties
            .get_string(PropertyCode::ResponseInformation)
            .unwrap();
        assert_eq!(data, REQUEST_RESPONSE_PREFIX_NAME);

        // RequestResponseInformation = 0
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(unique_id())
            .finalize();
        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::new_v5();
        let mut props = Properties::new();
        props
            .push_val(PropertyCode::RequestResponseInformation, 0u8)
            .unwrap();

        conn_opts
            .clean_session(false)
            .properties(props)
            .user_name(username())
            .password(password());
        let conn_opts = conn_opts.finalize();
        let resp = cli.connect(conn_opts).unwrap();
        let properties = resp.properties();
        assert!(properties
            .get_string(PropertyCode::ResponseInformation)
            .is_none());
    }

    #[tokio::test]
    async fn client5_request_response_test() {
        let network = "tcp";
        let qos = 1;
        let client_id = build_client_id(
            format!(
                "client5_request_response_test_{}_{}_{}",
                network,
                qos,
                unique_id()
            )
            .as_str(),
        );

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            request_response: true,
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let request_topic = format!("/{}/request/{}/{}", unique_id(), network, qos);
        let response_topic = format!("/{}/response/{}/{}", unique_id(), network, qos);
        let correlation_data = "correlation_data_123456".to_string();

        // publish data contain ResponseTopic&&CorrelationData
        let mut props: Properties = Properties::new();
        props
            .push_string(PropertyCode::ResponseTopic, response_topic.as_str())
            .unwrap();

        props
            .push_binary(
                PropertyCode::CorrelationData,
                serde_json::to_vec(&correlation_data).unwrap(),
            )
            .unwrap();

        let message_content =
            "client5_request_response_test message_content mqtt message".to_string();
        let msg = MessageBuilder::new()
            .properties(props)
            .topic(request_topic.clone())
            .payload(message_content.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        // subscribe
        let client_id = build_client_id(
            format!(
                "user_properties_test_request_{}_{}_{}",
                network,
                qos,
                unique_id()
            )
            .as_str(),
        );
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            request_response: true,
            ..Default::default()
        };
        let create_opts =
            build_create_conn_pros(&client_properties.client_id, &client_properties.addr);
        let cli = Client::new(create_opts).unwrap();
        let conn_opts = build_conn_pros(client_properties.clone(), client_properties.err_pwd);
        let result = cli.connect(conn_opts).unwrap();

        // get response topic prefix
        let response_topic_prefix = result
            .properties()
            .get_string(PropertyCode::ResponseInformation)
            .unwrap();

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            if payload != message_content {
                return false;
            }

            // get responseTopic && CorrelationData
            let topic = msg
                .properties()
                .get_string(PropertyCode::ResponseTopic)
                .unwrap();
            assert_eq!(topic, response_topic);

            let c_data: Vec<u8> = msg
                .properties()
                .get_binary(PropertyCode::CorrelationData)
                .unwrap();
            let c_data_1 = serde_json::from_slice::<String>(&c_data).unwrap();

            // push message to responseTopic
            let client_id = build_client_id(
                format!("user_properties_test_{}_{}_{}", network, qos, unique_id()).as_str(),
            );
            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(network),
                request_response: true,
                ..Default::default()
            };
            let cli = connect_server(&client_properties);
            let msg = MessageBuilder::new()
                .topic(format!("{}{}", response_topic_prefix, topic))
                .payload(c_data_1)
                .qos(qos)
                .finalize();

            publish_data(&cli, msg, false);
            distinct_conn(cli);
            true
        };
        subscribe_data_by_qos(&cli, &request_topic, qos, call_fn);
        distinct_conn(cli);

        // subscribe request topic
        let client_id = build_client_id(
            format!(
                "user_properties_test_response_{}_{}_{}",
                network,
                qos,
                unique_id()
            )
            .as_str(),
        );

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            request_response: true,
            ..Default::default()
        };

        let cli = connect_server(&client_properties);
        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == correlation_data
        };

        subscribe_data_by_qos(
            &cli,
            &format!("{}{}", response_topic_prefix, response_topic),
            qos,
            call_fn,
        );
        distinct_conn(cli);
    }
}
