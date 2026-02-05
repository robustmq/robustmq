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
    use paho_mqtt::{Message, SubscribeOptions};

    #[tokio::test]
    async fn retain_as_published() {
        for retain_as_published in [true, false] {
            let subscribe_options = SubscribeOptions::new(false, retain_as_published, None);
            let network = "tcp";
            let qos = 1;
            let uid = unique_id();
            let topic = format!("/retain_as_published/{uid}/{network}/{qos}");

            // publish
            let client_id = build_client_id(format!("retain_as_published{uid}").as_str());
            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(network),
                ..Default::default()
            };
            let cli = connect_server(&client_properties);

            let message_content = "retain message".to_string();
            let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
            publish_data(&cli, msg, false);

            let call_fn = |msg: Message| {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                if payload != message_content {
                    return false;
                }

                msg.retained() == retain_as_published
            };

            let subscribe_test_data = SubscribeTestData {
                sub_topic: topic,
                sub_qos: qos,
                subscribe_options,
                subscribe_properties: None,
            };
            let res = subscribe_data_with_options(&cli, subscribe_test_data, call_fn).await;
            assert!(res.is_ok(), "subscribe_data_with_options failed: {:?}", res);

            distinct_conn(cli);
        }
    }
}
