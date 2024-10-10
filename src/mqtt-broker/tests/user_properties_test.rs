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
    async fn user_properties_test() {
        let client_id = unique_id();
        let addr = broker_addr();
        let topic = format!("/tests/{}", unique_id());
        let sub_topics = &[topic.clone()];

        let cli = connect_server5(&client_id, &addr, false, false);
        let message_content = "mqtt message".to_string();

        // publish
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

        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(message_content.clone())
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
                if payload == message_content {
                    assert!(true);
                } else {
                    assert!(false);
                }
                let user_properties = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 0)
                    .unwrap();
                assert_eq!(user_properties.0, "age".to_string());
                assert_eq!(user_properties.1, "1".to_string());

                let user_properties = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 1)
                    .unwrap();
                assert_eq!(user_properties.0, "name".to_string());
                assert_eq!(user_properties.1, "robustmq".to_string());

                break;
            } else {
                assert!(false);
            }
        }
        distinct_conn(cli);
    }
}
