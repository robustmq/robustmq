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
    use std::collections::HashMap;

    use common_base::tools::unique_id;
    use paho_mqtt::{Message, QOS_1};

    use crate::mqtt_protocol::common::{
        broker_wss_addr, connect_server34, connect_server5, distinct_conn,
    };

    #[tokio::test]
    async fn wss3_sub_pub_test() {
        let client_id = unique_id();
        let addr = broker_wss_addr();
        let pub_topic = unique_id();
        let cli = connect_server34(4, &client_id, &addr, true, true);

        let mut data = HashMap::new();
        data.insert("v1", "lobo");
        data.insert("v2", "lobo1");
        let message_content = serde_json::to_string(&data).unwrap();

        // publish
        let msg = Message::new(pub_topic.clone(), message_content.clone(), QOS_1);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let sub_qos = &[1];
        let sub_topics = &[pub_topic.clone()];

        // sub
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }
        if let Some(msg) = rx.iter().next() {
            let msg = msg.unwrap();
            println!("{}", msg);
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            assert_eq!(payload, message_content);
        }
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn wss4_sub_pub_test() {
        let client_id = unique_id();
        let addr = broker_wss_addr();
        let pub_topic = unique_id();

        let cli = connect_server34(4, &client_id, &addr, true, true);

        let mut data = HashMap::new();
        data.insert("v1", "lobo");
        data.insert("v2", "lobo1");
        let message_content = serde_json::to_string(&data).unwrap();

        // publish
        let msg = Message::new(pub_topic.clone(), message_content.clone(), QOS_1);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let sub_qos = &[1];
        let sub_topics = &[pub_topic.clone()];

        // sub
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }
        if let Some(msg) = rx.iter().next() {
            let msg = msg.unwrap();
            println!("{}", msg);
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            assert_eq!(payload, message_content);
        }
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn wss5_sub_pub_test() {
        let client_id = unique_id();
        let addr = broker_wss_addr();
        let pub_topic = unique_id();

        let cli = connect_server5(&client_id, &addr, true, true);

        let mut data = HashMap::new();
        data.insert("v1", "lobo");
        data.insert("v2", "lobo1");
        let message_content = serde_json::to_string(&data).unwrap();

        // publish
        let msg = Message::new(pub_topic.clone(), message_content.clone(), QOS_1);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let sub_qos = &[1];
        let sub_topics = &[pub_topic.clone()];

        // sub
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }
        if let Some(msg) = rx.iter().next() {
            let msg = msg.unwrap();
            println!("{}", msg);
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            assert_eq!(payload, message_content);
        }
        distinct_conn(cli);
    }
}
