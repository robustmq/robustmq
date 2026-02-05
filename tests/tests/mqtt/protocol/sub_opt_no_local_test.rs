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

// it has a problem from client, it will block in test case
// I didn't know why
#[cfg(test)]
mod tests {
    use crate::mqtt::protocol::common::{
        broker_addr_by_type, build_client_id, connect_server, distinct_conn, publish_data,
        subscribe_data_with_options, SubscribeTestData,
    };
    use crate::mqtt::protocol::ClientTestProperties;
    use common_base::uuid::unique_id;
    use paho_mqtt::{Message, MessageBuilder, SubscribeOptions};

    #[tokio::test]
    async fn no_local_is_true() {
        let subscribe_options = SubscribeOptions::new(true, false, None);
        let network = "tcp";
        let qos = 1;
        let uid = unique_id();
        let topic = format!("/no_local_is_true/{uid}/{network}/{qos}");
        let client_id = build_client_id(format!("no_local_is_true_{uid}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message_content = "no_local_is_true content".to_string();
        let msg = MessageBuilder::new()
            .payload(message_content.clone())
            .topic(topic.clone())
            .qos(qos)
            .retained(false)
            .finalize();
        publish_data(&cli, msg, false);

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message_content
        };

        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };
        let res = subscribe_data_with_options(&cli, subscribe_test_data, call_fn).await;
        assert!(res.is_err(), "Expected timeout but got: {:?}", res);
    }

    #[tokio::test]
    async fn no_local_is_false() {
        let subscribe_options = SubscribeOptions::new(false, false, None);
        let network = "tcp";
        let qos = 1;
        let uid = unique_id();
        let topic = format!("/no_local_is_false/{uid}/{network}/{qos}");

        // publish
        let client_id = build_client_id(format!("no_local_is_false{uid}").as_str());
        let client_test_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };

        let cli = connect_server(&client_test_properties);
        let message_content = "construct topic".to_string();
        let msg = MessageBuilder::new()
            .payload(message_content.clone())
            .topic(topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        // subscribe
        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            payload == message_content
        };

        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };

        let res = subscribe_data_with_options(&cli, subscribe_test_data, call_fn).await;
        assert!(res.is_ok(), "subscribe_data_with_options failed: {:?}", res);
        distinct_conn(cli);
    }
}
