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
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, QOS_1};

    use crate::mqtt_protocol::common::{broker_addr, connect_server5, distinct_conn};

    #[tokio::test]
    async fn client5_subscribe_test() {
        let sub_qos = &[0];
        let topic = format!("/tests/{}", unique_id());
        let sub_topic = format!("$share/g1{}", topic);
        simple_test(topic.clone(), sub_topic.clone(), sub_qos, "2".to_string()).await;

        let sub_qos = &[1];
        let topic = format!("/tests/{}", unique_id());
        let sub_topic = format!("$share/g1{}", topic);
        simple_test(topic.clone(), sub_topic.clone(), sub_qos, "1".to_string()).await;

        let sub_qos = &[2];
        let topic = format!("/tests/{}", unique_id());
        let sub_topic = format!("$share/g1{}", topic);
        simple_test(topic.clone(), sub_topic.clone(), sub_qos, "3".to_string()).await;
    }

    async fn simple_test(
        pub_topic: String,
        sub_topic: String,
        sub_qos: &[i32],
        payload_flag: String,
    ) {
        let client_id = unique_id();
        let addr = broker_addr();
        let sub_topics = &[sub_topic.clone()];

        let cli = connect_server5(&client_id, &addr, false, false);
        let message_content = format!("mqtt {payload_flag} message");

        // publish
        let msg = Message::new(pub_topic.clone(), message_content.clone(), QOS_1);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
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
        if let Some(msg) = rx.iter().next() {
            let msg = msg.unwrap();
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            assert_eq!(payload, message_content);
        }
        distinct_conn(cli);
    }
}
