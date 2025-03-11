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
    use mqtt_broker::handler::constant::{
        SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE,
    };
    use paho_mqtt::{MessageBuilder, PropertyCode, QOS_1};

    use crate::mqtt_protocol::common::{broker_addr, connect_server5, distinct_conn};

    #[tokio::test]
    async fn retain_message_sub_qos0_test() {
        let client_id = unique_id();
        let addr = broker_addr();
        let topic = format!("/tests/{}", unique_id());
        let sub_topics = &[topic.clone()];

        let cli = connect_server5(&client_id, &addr, false, false);
        let message_content = "mqtt message".to_string();

        let msg = MessageBuilder::new()
            .payload(message_content.clone())
            .topic(topic.clone())
            .qos(QOS_1)
            .retained(true)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        distinct_conn(cli);

        // subscribe
        let client_id = unique_id();
        let cli = connect_server5(&client_id, &addr, false, false);
        let sub_qos = &[0];
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        for msg in rx.iter() {
            let msg = msg.unwrap();
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            println!("{}", payload.clone());
            if payload == message_content {
                if let Some(raw) = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 0)
                {
                    if raw.0 == *SUB_RETAIN_MESSAGE_PUSH_FLAG
                        && raw.1 == *SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE
                    {
                        break;
                    }
                }
            }
        }
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn retain_message_sub_qos1_test() {
        let client_id = unique_id();
        let addr = broker_addr();
        let topic = format!("/tests/{}", unique_id());
        let sub_topics = &[topic.clone()];

        let cli = connect_server5(&client_id, &addr, false, false);
        let message_content = "mqtt message".to_string();

        let msg = MessageBuilder::new()
            .payload(message_content.clone())
            .topic(topic.clone())
            .qos(QOS_1)
            .retained(true)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        distinct_conn(cli);

        // subscribe
        let client_id = unique_id();
        let cli = connect_server5(&client_id, &addr, false, false);
        let sub_qos = &[1];
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        for msg in rx.iter() {
            let msg = msg.unwrap();
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            println!("{}", payload.clone());
            if payload == message_content {
                if let Some(raw) = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 0)
                {
                    if raw.0 == *SUB_RETAIN_MESSAGE_PUSH_FLAG
                        && raw.1 == *SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE
                    {
                        break;
                    }
                }
            }
        }
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn retain_message_sub_qos2_test() {
        let client_id = unique_id();
        let addr = broker_addr();
        let topic = format!("/tests/{}", unique_id());
        let sub_topics = &[topic.clone()];

        let cli = connect_server5(&client_id, &addr, false, false);
        let message_content = "mqtt message".to_string();

        let msg = MessageBuilder::new()
            .payload(message_content.clone())
            .topic(topic.clone())
            .qos(QOS_1)
            .retained(true)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        distinct_conn(cli);

        // subscribe
        let client_id = unique_id();
        let cli = connect_server5(&client_id, &addr, false, false);
        let sub_qos = &[2];
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        for msg in rx.iter() {
            let msg = msg.unwrap();
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            println!("{}", payload.clone());
            if payload == message_content {
                if let Some(raw) = msg
                    .properties()
                    .get_string_pair_at(PropertyCode::UserProperty, 0)
                {
                    if raw.0 == *SUB_RETAIN_MESSAGE_PUSH_FLAG
                        && raw.1 == *SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE
                    {
                        break;
                    }
                }
            }
        }
        distinct_conn(cli);
    }
}
