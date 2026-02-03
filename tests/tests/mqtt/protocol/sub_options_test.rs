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
        ssl_by_type, subscribe_data_with_options, ws_by_type, SubscribeTestData,
    };
    use crate::mqtt::protocol::ClientTestProperties;
    use common_base::uuid::unique_id;
    use mqtt_broker::core::constant::{
        SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE,
    };
    use paho_mqtt::{Message, MessageBuilder, PropertyCode, RetainHandling, SubscribeOptions};
    use std::time::Duration;

    #[tokio::test]
    async fn no_local_is_true() {
        let subscribe_options = SubscribeOptions::new(true, false, None);
        let network = "tcp";
        let qos = 1;
        let uid = unique_id();
        let topic =
            format!("/mqtt5_should_not_recv_msg_when_no_local_is_true/{uid}/{network}/{qos}");
        let client_id = build_client_id(format!("no_local_is_true_{uid}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
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

        let receiver = cli.start_consuming();
        assert!(cli
            .subscribe_with_options(&topic, qos, subscribe_options, None)
            .is_ok());

        let timeout_fn = async || -> bool {
            loop {
                let res = receiver.recv_timeout(Duration::from_secs(5));
                if let Ok(msg_opt) = &res {
                    println!("{res:?}");
                    let msg = msg_opt.as_ref().unwrap();
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    if payload == message_content {
                        return true;
                    }
                    continue;
                }
                return false;
            }
        };
        assert!(!timeout_fn().await);
        distinct_conn(cli);
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
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
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

        subscribe_data_with_options(&cli, subscribe_test_data, call_fn);
        distinct_conn(cli);
    }

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
                ws: ws_by_type(network),
                ssl: ssl_by_type(network),
                ..Default::default()
            };
            let cli = connect_server(&client_properties);

            let message_content = "retain message".to_string();
            let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
            publish_data(&cli, msg, false);

            let call_fn = |msg: Message| {
                println!(
                    "msg: {:?},retained:{},retain_as_published:{}",
                    msg,
                    msg.retained(),
                    retain_as_published
                );
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
            subscribe_data_with_options(&cli, subscribe_test_data, call_fn);

            distinct_conn(cli);
        }
    }

    #[ignore]
    #[tokio::test]
    async fn retain_handling_is_0() {
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
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message_content = "retain message".to_string();
        let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        // sub new
        let sub_cli = build_client_id(format!("retain_handling_is_0{network}_{qos}").as_str());
        let sub_cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: sub_cli.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
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

        subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);

        // sub old
        assert!(sub_cli.unsubscribe(&topic).is_ok());

        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };

        subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);
        distinct_conn(sub_cli);
    }

    #[ignore = "reason"]
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
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message_content = "retain message".to_string();
        let msg = Message::new_retained(topic.clone(), message_content.clone(), qos);
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        // sub new
        let sub_cli = build_client_id(format!("handling_is_1{network}_{qos}").as_str());
        let sub_cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: sub_cli.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
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

        subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);

        // sub old
        assert!(sub_cli.unsubscribe(&topic).is_ok());

        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            if payload != message_content {
                return false;
            }
            msg.properties()
                .get_string_pair_at(PropertyCode::UserProperty, 0)
                .is_none()
        };

        let subscribe_test_data = SubscribeTestData {
            sub_topic: topic.clone(),
            sub_qos: qos,
            subscribe_options,
            subscribe_properties: None,
        };

        subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);
        distinct_conn(sub_cli);
    }

    #[ignore = "reason"]
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
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message_content = "retain message".to_string();
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
        subscribe_data_with_options(&sub_cli, subscribe_test_data, call_fn);
        distinct_conn(sub_cli);
    }
}
