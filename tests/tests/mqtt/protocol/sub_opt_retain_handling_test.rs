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
    use std::time::Duration;

    use crate::mqtt::protocol::common::{
        broker_addr_by_type, build_client_id, connect_server, distinct_conn, publish_data,
        ssl_by_type, subscribe_data_with_options, ws_by_type, SubscribeTestData,
    };
    use crate::mqtt::protocol::ClientTestProperties;
    use common_base::uuid::unique_id;
    use mqtt_broker::core::constant::{
        SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE,
    };
    use paho_mqtt::{Message, PropertyCode, RetainHandling, SubscribeOptions};
    use tokio::time::sleep;

    #[tokio::test]
    async fn retain_handling_0() {
        let subscribe_options =
            SubscribeOptions::new(false, false, RetainHandling::SendRetainedOnSubscribe);
        let network = "tcp";
        let qos = 1;
        let uid = unique_id();
        let topic = format!("/retain_handling_is_0/{uid}/{network}/{qos}");

        // publish
        let client_id = build_client_id(format!("retain_handling_is_0{uid}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message_content = "retain_handling_is_0 retain message".to_string();
        let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        // sub new
        let sub_cli = build_client_id(format!("retain_handling_is_0{network}_{qos}").as_str());
        let sub_cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: sub_cli.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        });

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            if payload != message_content {
                return false;
            }
            if msg
                .properties()
                .get_string_pair_at(PropertyCode::UserProperty, 0)
                .is_some()
            {
                let raw = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 0)
                    .unwrap();
                if raw.0 == *SUB_RETAIN_MESSAGE_PUSH_FLAG
                    || raw.1 == *SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE
                {
                    return true;
                }
            }
            false
        };

        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };

        let res = subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn).await;
        assert!(res.is_ok(), "subscribe_data_with_options failed: {:?}", res);

        // sub old
        assert!(sub_cli.unsubscribe(&topic).is_ok());

        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };

        let res = subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn).await;
        assert!(res.is_ok(), "subscribe_data_with_options failed: {:?}", res);
        distinct_conn(sub_cli);
    }

    #[tokio::test]
    async fn handling_is_1() {
        let subscribe_options =
            SubscribeOptions::new(false, false, RetainHandling::SendRetainedOnNew);
        let network = "tcp";
        let qos = 1;
        let uid = unique_id();
        let topic = format!("/handling_is_1/{uid}/{network}/{qos}");

        // publish
        let client_id = build_client_id(format!("handling_is_1{uid}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message_content = "handling_is_1 retain message".to_string();
        let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        // sub new
        let sub_cli = build_client_id(format!("handling_is_1{network}_{qos}").as_str());
        let sub_cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: sub_cli.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        });

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            if payload != message_content {
                return false;
            }
            if msg
                .properties()
                .get_string_pair_at(PropertyCode::UserProperty, 0)
                .is_some()
            {
                let raw = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 0)
                    .unwrap();
                if raw.0 == *SUB_RETAIN_MESSAGE_PUSH_FLAG
                    || raw.1 == *SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE
                {
                    return true;
                }
            }
            false
        };

        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };

        let res = subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn).await;
        assert!(res.is_ok(), "subscribe_data_with_options failed: {:?}", res);

        sleep(Duration::from_secs(3)).await;
        // sub old
        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };

        let res = subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn).await;
        assert!(
            res.is_err(),
            "subscribe_data_with_options  1 failed: {:?}",
            res
        );
        distinct_conn(sub_cli);
    }

    #[tokio::test]
    async fn handling_is_2() {
        let subscribe_options =
            SubscribeOptions::new(false, false, RetainHandling::DontSendRetained);
        let network = "tcp";
        let qos = 1;
        let uid = unique_id();
        let topic = format!("/handling_is_2/{uid}/{network}/{qos}");

        // publish
        let client_id = build_client_id(format!("handling_is_2{uid}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message_content = "handling_is_2 retain message".to_string();
        let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        // sub
        let sub_client_id = build_client_id(format!("handling_is_2{network}_{qos}").as_str());
        let sub_cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: sub_client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        });

        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            if payload != message_content {
                return false;
            }

            let user_properties = msg
                .properties()
                .get_string_pair_at(PropertyCode::UserProperty, 0);

            user_properties.is_none()
        };
        let res = subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn).await;
        assert!(res.is_ok(), "subscribe_data_with_options failed: {:?}", res);
        distinct_conn(sub_cli);
    }
}
