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

mod common;

#[cfg(test)]
mod tests {
    use crate::common::{broker_addr, connect_server5, distinct_conn};
    use common_base::tools::unique_id;
    use paho_mqtt::{MessageBuilder, Properties, PropertyCode, SubscribeOptions, QOS_1};

    #[tokio::test]
    async fn client5_sub_identifier_test() {
        let sub_qos = &[0, 0];

        let topic = unique_id();

        let topic1 = format!("/test/{}/+", topic);
        let topic2 = format!("/test/{}/test", topic);
        let topic3 = format!("/test/{}/test_one", topic);
        simple_test(topic1, topic2, topic3, sub_qos, "2".to_string()).await;
    }

    async fn simple_test(
        topic1: String,
        topic2: String,
        topic3: String,
        sub_qos: &[i32],
        payload_flag: String,
    ) {
        let client_id = unique_id();
        let addr = broker_addr();
        let cli = connect_server5(&client_id, &addr);

        let message_content = format!("mqtt {payload_flag} message");
        let msg = MessageBuilder::new()
            .topic(topic2.clone())
            .payload(message_content.clone())
            .qos(QOS_1)
            .finalize();

        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }

        // subscribe
        let mut props: Properties = Properties::new();
        props.push_int(PropertyCode::SubscriptionIdentifier, 1).unwrap();
        match cli.subscribe_many_with_options(
            &[topic1.as_str()],
            &[sub_qos[0]],
            &[SubscribeOptions::default()],
            Some(props),
        ) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        let mut props: Properties = Properties::new();
        props.push_int(PropertyCode::SubscriptionIdentifier, 2).unwrap();
        match cli.subscribe_many_with_options(
            &[topic2.as_str()],
            &[sub_qos[1]],
            &[SubscribeOptions::default()],
            Some(props),
        ) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        let rx = cli.start_consuming();
        let mut msgs = rx.iter();

        let mut r_one = false;
        let mut r_two = false;

        if let Some(Some(msg)) = msgs.next() {
            let sub_identifier =
                msg.properties().get_int(PropertyCode::SubscriptionIdentifier).unwrap();
            println!("{:?} sub_identifier: {}", msg, sub_identifier);
            match sub_identifier {
                1 => {
                    r_one = true;
                }
                2 => {
                    r_two = true;
                }
                _ => {
                    assert!(false);
                }
            }
        } else {
            assert!(false);
        }

        if let Some(Some(msg)) = msgs.next() {
            let sub_identifier =
                msg.properties().get_int(PropertyCode::SubscriptionIdentifier).unwrap();
            println!("{:?} sub_identifier: {}", msg, sub_identifier);
            match sub_identifier {
                1 => {
                    r_one = true;
                }
                2 => {
                    r_two = true;
                }
                _ => {
                    assert!(false);
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(r_one, r_two);

        let msg = MessageBuilder::new()
            .topic(topic3)
            .payload(message_content.clone())
            .qos(QOS_1)
            .finalize();

        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }

        if let Some(Some(msg)) = msgs.next() {
            let sub_identifier =
                msg.properties().get_int(PropertyCode::SubscriptionIdentifier).unwrap();
            assert_eq!(sub_identifier, 1);

            println!("{msg:?}");
            println!("{sub_identifier:?}");
        } else {
            assert!(false);
        }

        distinct_conn(cli);
    }
}
