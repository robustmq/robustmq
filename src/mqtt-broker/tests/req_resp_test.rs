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
    use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode, QOS_1};

    #[tokio::test]
    async fn client5_reuqset_response_test() {
        let sub_qos = &[0, 0];
        let topic = unique_id();
        let requset_topic = format!("/request/{}", topic);
        let response_topic = format!("/response/{}", topic);
        simple_test(requset_topic, response_topic, sub_qos, "2".to_string()).await;

        let sub_qos = &[1, 1];
        let topic = unique_id();
        let requset_topic = format!("/request/{}", topic);
        let response_topic = format!("/response/{}", topic);
        simple_test(requset_topic, response_topic, sub_qos, "1".to_string()).await;

        let sub_qos = &[2, 2];
        let topic = unique_id();
        let requset_topic = format!("/request/{}", topic);
        let response_topic = format!("/response/{}", topic);
        simple_test(requset_topic, response_topic, sub_qos, "3".to_string()).await;
    }

    async fn simple_test(
        requset_topic: String,
        response_topic: String,
        sub_qos: &[i32],
        payload_flag: String,
    ) {
        let client_id = unique_id();
        let addr = broker_addr();
        let sub_topics = &[requset_topic.clone(), response_topic.clone()];

        let cli = connect_server5(&client_id, &addr);

        let message_content = format!("mqtt {payload_flag} message");

        // publish
        let mut props: Properties = Properties::new();
        props
            .push_string(PropertyCode::ResponseTopic, response_topic.as_str())
            .unwrap();
        let msg = MessageBuilder::new()
            .properties(props)
            .topic(requset_topic.clone())
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
                // println!("received message: {:?}", msg);

                if msg.topic() == requset_topic {
                    let topic = msg
                        .properties()
                        .get_string(PropertyCode::ResponseTopic)
                        .unwrap();
                    assert_eq!(topic, response_topic);

                    let msg = Message::new(topic, payload.clone(), QOS_1);
                    match cli.publish(msg) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("{}", e);
                            assert!(false);
                        }
                    }
                    continue;
                }

                if payload == message_content {
                    assert!(true);
                } else {
                    assert!(false);
                }
                break;
            } else {
                assert!(false);
            }
        }

        assert!(true);
        distinct_conn(cli);
    }
}
