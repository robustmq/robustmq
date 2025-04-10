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
    use crate::mqtt_protocol::common::{
        broker_addr_by_type, build_client_id, connect_server, network_types, publish_data,
        qos_list, ssl_by_type, subscribe_data_with_options, ws_by_type, SubscribeTestData,
    };
    use crate::mqtt_protocol::ClientTestProperties;
    use common_base::tools::unique_id;
    use mqtt_broker::handler::constant::{
        SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE,
    };
    use paho_mqtt::{Message, MessageBuilder, PropertyCode, RetainHandling, SubscribeOptions};
    use std::cell::RefCell;
    use std::time::Duration;

    #[tokio::test]
    async fn mqtt5_should_not_recv_msg_when_no_local_is_true() {
        let subscribe_options = SubscribeOptions::new(true, false, None);
        for network in network_types() {
            for qos in qos_list() {
                let uid = unique_id();
                let topic = format!("/no_local/{}/{}/{}", uid, network, qos);
                let client_id = build_client_id(format!("no_local_client_{}", uid).as_str());
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);
                let message_content = "construct topic".to_string();
                let msg = MessageBuilder::new()
                    .payload(message_content.clone())
                    .topic(topic.clone())
                    .qos(qos)
                    .retained(false)
                    .finalize();
                publish_data(&cli, msg, false);

                let receiver = cli.start_consuming();

                assert!(cli
                    .subscribe_with_options(&topic, qos, subscribe_options, None)
                    .is_ok());

                let message_content = "mqtt message".to_string();
                let msg = MessageBuilder::new()
                    .payload(message_content.clone())
                    .topic(topic.clone())
                    .qos(qos)
                    .retained(false)
                    .finalize();
                assert!(cli.publish(msg).is_ok());

                let mut is_no_local = true;
                if let Ok(Some(msg)) = receiver.recv_timeout(Duration::from_secs(5)) {
                    is_no_local = false;
                    assert_eq!(
                        String::from_utf8(msg.payload().to_vec()).unwrap(),
                        message_content
                    )
                };
                assert!(is_no_local);
            }
        }
    }

    // if we want to test the no_local in a new topic,
    // we need to publish a message and then subscribe the topic
    // then we can
    #[tokio::test]
    async fn mqtt5_should_recv_msg_when_no_local_is_false() {
        let subscribe_options = SubscribeOptions::new(false, false, None);

        for network in network_types() {
            for qos in qos_list() {
                let uid = unique_id();
                let topic = format!("/local_test/{}/{}/{}", uid, network, qos);
                let client_id = build_client_id(format!("local_client_{}", uid).as_str());

                let client_test_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };

                let cli = connect_server(&client_test_properties);
                let message_content = "construct topic".to_string();
                let msg = MessageBuilder::new()
                    .payload(message_content.clone())
                    .topic(topic.clone())
                    .qos(qos)
                    .retained(false)
                    .finalize();
                publish_data(&cli, msg, false);

                let is_no_local = RefCell::new(true);
                let call_fn = |msg: Message| {
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    if payload != message_content {
                        return false;
                    }
                    *is_no_local.borrow_mut() = false;
                    true
                };

                let subscribe_test_data = SubscribeTestData {
                    sub_topic: topic.clone(),
                    sub_qos: qos,
                    subscribe_options,
                    subscribe_properties: None,
                };

                subscribe_data_with_options(&cli, subscribe_test_data, call_fn);

                assert!(!*is_no_local.borrow())
            }
        }
    }

    #[tokio::test]
    async fn mqtt5_should_recv_retain_message_with_retain_as_published() {
        for retain_as_published in [true, false] {
            let subscribe_options = SubscribeOptions::new(false, retain_as_published, None);
            for network in network_types() {
                for qos in qos_list() {
                    let uid = unique_id();
                    let topic = format!("/retain_as_published/{}/{}/{}", uid, network, qos);
                    let client_id =
                        build_client_id(format!("retain_as_published_{}", uid).as_str());
                    let client_properties = ClientTestProperties {
                        mqtt_version: 5,
                        client_id: client_id.to_string(),
                        addr: broker_addr_by_type(&network),
                        ws: ws_by_type(&network),
                        ssl: ssl_by_type(&network),
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
                        let test_retain_as_published = retain_as_published;
                        if msg.retained() != test_retain_as_published {
                            return false;
                        }
                        true
                    };

                    let subscribe_test_data = SubscribeTestData {
                        sub_topic: topic,
                        sub_qos: qos,
                        subscribe_options,
                        subscribe_properties: None,
                    };
                    subscribe_data_with_options(&cli, subscribe_test_data, call_fn);
                }
            }
        }
    }

    #[tokio::test]
    async fn mqtt5_should_recv_retain_message_every_subscribe_when_retain_handling_is_0() {
        let subscribe_options =
            SubscribeOptions::new(false, false, RetainHandling::SendRetainedOnSubscribe);
        for network in network_types() {
            for qos in qos_list() {
                let uid = unique_id();
                let topic = format!("/retain_handling_0/{}/{}/{}", uid, network, qos);
                let client_id = build_client_id(format!("retain_handling_0_{}", uid).as_str());
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                let message_content = "retain message".to_string();
                let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
                publish_data(&cli, msg, false);

                let sub_cli = build_client_id(
                    format!("retain_handling_sub_0_test_{}_{}", network, qos).as_str(),
                );
                let sub_cli = connect_server(&ClientTestProperties {
                    mqtt_version: 5,
                    client_id: sub_cli.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
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
                        if raw.0 != *SUB_RETAIN_MESSAGE_PUSH_FLAG
                            || raw.1 != *SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE
                        {
                            return false;
                        }
                    }
                    true
                };

                let subscribe_test_data = SubscribeTestData {
                    sub_topic: topic.clone(),
                    sub_qos: qos,
                    subscribe_options,
                    subscribe_properties: None,
                };

                subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);

                assert!(sub_cli.unsubscribe(&topic).is_ok());

                let subscribe_test_data = SubscribeTestData {
                    sub_topic: topic.clone(),
                    sub_qos: qos,
                    subscribe_options,
                    subscribe_properties: None,
                };

                subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);
            }
        }
    }

    #[tokio::test]
    async fn mqtt5_should_not_recv_retain_message_new_subscribe_when_retain_handling_is_1() {
        let subscribe_options =
            SubscribeOptions::new(false, false, RetainHandling::SendRetainedOnNew);

        for network in network_types() {
            for qos in qos_list() {
                let uid = unique_id();
                let topic = format!("/retain_handling_1/{}/{}/{}", uid, network, qos);
                let client_id = build_client_id(format!("retain_handling_1_{}", uid).as_str());
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                let sub_cli = build_client_id(
                    format!("retain_handling_sub_1_test_{}_{}", network, qos).as_str(),
                );
                let sub_cli = connect_server(&ClientTestProperties {
                    mqtt_version: 5,
                    client_id: sub_cli.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                });

                let message_content = "retain message".to_string();
                let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
                publish_data(&cli, msg, false);

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
                        if raw.0 != *SUB_RETAIN_MESSAGE_PUSH_FLAG
                            || raw.1 != *SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE
                        {
                            return false;
                        }
                    }
                    true
                };

                let subscribe_test_data = SubscribeTestData {
                    sub_topic: topic.clone(),
                    sub_qos: qos,
                    subscribe_options,
                    subscribe_properties: None,
                };

                subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);

                assert!(sub_cli.unsubscribe(&topic).is_ok());

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
                        return false;
                    }
                    true
                };

                let subscribe_test_data = SubscribeTestData {
                    sub_topic: topic.clone(),
                    sub_qos: qos,
                    subscribe_options,
                    subscribe_properties: None,
                };

                subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);
            }
        }
    }

    #[tokio::test]
    async fn mqtt5_should_not_recv_retain_message_every_subscribe_when_retain_handling_is_2() {
        let subscribe_options =
            SubscribeOptions::new(false, false, RetainHandling::DontSendRetained);
        for network in network_types() {
            for qos in qos_list() {
                let uid = unique_id();
                let topic = format!("/retain_handling_2/{}/{}/{}", uid, network, qos);
                let client_id = build_client_id(format!("retain_handling_2_{}", uid).as_str());
                let client_properties = ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                let cli = connect_server(&client_properties);

                let message_content = "retain message".to_string();
                let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
                publish_data(&cli, msg, false);

                let sub_cli = build_client_id(
                    format!("retain_handling_sub_2_test_{}_{}", network, qos).as_str(),
                );
                let sub_cli = connect_server(&ClientTestProperties {
                    mqtt_version: 5,
                    client_id: sub_cli.to_string(),
                    addr: broker_addr_by_type(&network),
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
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
                    if msg
                        .properties()
                        .get_string_pair_at(PropertyCode::UserProperty, 0)
                        .is_some()
                    {
                        return false;
                    }
                    true
                };
                subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);
            }
        }
    }
}
