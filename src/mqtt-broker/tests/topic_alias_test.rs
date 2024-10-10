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
    use common_base::tools::unique_id;
    use paho_mqtt::{MessageBuilder, Properties, PropertyCode, QOS_1};

    use crate::common::{broker_addr, connect_server5, distinct_conn};

    #[tokio::test]
    async fn topic_alias_test() {
        let client_id = unique_id();
        let addr = broker_addr();
        let topic_alias = 1;
        let topic = format!("/tests/{}", unique_id());
        let sub_topics = &[topic.clone()];

        let cli = connect_server5(&client_id, &addr, false, false);
        let message_content1 = "mqtt message".to_string();

        // publish to topic
        let mut props = Properties::new();
        props
            .push_u32(PropertyCode::MessageExpiryInterval, 50)
            .unwrap();
        props
            .push_string_pair(PropertyCode::UserProperty, "age", "1")
            .unwrap();
        props
            .push_string_pair(PropertyCode::UserProperty, "name", "robustmq")
            .unwrap();
        props
            .push_u16(PropertyCode::TopicAlias, topic_alias)
            .unwrap();

        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(message_content1.clone())
            .topic(topic.clone())
            .qos(QOS_1)
            .retained(false)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }

        let message_content2 = "mqtt message alias".to_string();

        // publish to topic alias
        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(message_content2.clone())
            .qos(QOS_1)
            .retained(false)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }

        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(message_content2.clone())
            .qos(QOS_1)
            .retained(false)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }

        props.push_u16(PropertyCode::TopicAlias, 2).unwrap();
        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(message_content2.clone())
            .qos(QOS_1)
            .retained(false)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {
                assert!(false);
            }
            Err(_) => {
                assert!(true);
            }
        }

        // subscribe
        let sub_qos = &[1];
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }
        for msg in rx.iter() {
            if let Some(msg) = msg {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                println!("recv message: {}", payload);
                if payload == message_content2 {
                    assert!(true);
                    break;
                }
            } else {
                assert!(false);
            }
        }
        distinct_conn(cli);
    }
}
