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
    use std::time::Duration;

    use common_base::tools::unique_id;
    use mqtt_broker::handler::constant::{
        SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE,
    };
    use paho_mqtt::{Message, PropertyCode, RetainHandling, SubscribeOptions, QOS_0, QOS_2};

    use crate::mqtt_protocol::common::{
        broker_addr, build_client_id, connect_server5, distinct_conn,
    };

    #[tokio::test]
    async fn sub_options_no_local_test() {
        let topic = format!("/tests/{}", unique_id());
        no_local_test(topic.clone(), topic.clone(), true, "1".to_string()).await;

        let topic = format!("/tests/{}", unique_id());
        no_local_test(topic.clone(), topic.clone(), false, "0".to_string()).await;
    }

    #[tokio::test]
    async fn sub_options_retain_as_published_test() {
        let topic = format!("/tests/{}", unique_id());
        retain_as_published_test(topic.clone(), topic.clone(), true, "1".to_string()).await;

        let topic = format!("/tests/{}", unique_id());
        retain_as_published_test(topic.clone(), topic.clone(), false, "0".to_string()).await;
    }

    async fn no_local_test(
        pub_topic: String,
        sub_topic: String,
        no_local: bool,
        payload_flag: String,
    ) {
        let client_id = build_client_id("no_local_test");
        let addr = broker_addr();
        let sub_topics = &[sub_topic.clone()];
        let sub_qos = &[QOS_2];
        let sub_opts = &[SubscribeOptions::new(no_local, false, None)];

        let cli = connect_server5(&client_id, &addr, false, false);

        let message_content = format!("mqtt {payload_flag} message");

        // publish
        let msg = Message::new(pub_topic.clone(), message_content.clone(), QOS_2);
        assert!(cli.publish(msg).is_ok());

        // subscribe
        let rx = cli.start_consuming();
        assert!(cli
            .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
            .is_ok());

        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(msg) => {
                assert!(!no_local);
                let payload = String::from_utf8(msg.unwrap().payload().to_vec()).unwrap();
                assert_eq!(payload, message_content);
            }
            Err(_) => {
                assert!(no_local);
            }
        };
        distinct_conn(cli);
    }

    async fn retain_as_published_test(
        pub_topic: String,
        sub_topic: String,
        retain_as_published: bool,
        payload_flag: String,
    ) {
        let client_id = build_client_id("retain_as_published_test");
        let addr = broker_addr();
        let sub_topics = &[sub_topic.clone()];
        let sub_qos = &[QOS_0];
        let sub_opts = &[SubscribeOptions::new(false, retain_as_published, None)];

        let cli = connect_server5(&client_id, &addr, false, false);

        let message_content = format!("mqtt {payload_flag} message");
        // publish
        let msg = Message::new_retained(pub_topic.clone(), message_content.clone(), QOS_0);
        assert!(cli.publish(msg).is_ok());

        // subscribe
        let rx = cli.start_consuming();
        assert!(cli
            .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
            .is_ok());
        let recv = rx.recv_timeout(Duration::from_secs(60));
        assert!(recv.is_ok());
        assert_eq!(recv.unwrap().unwrap().retained(), retain_as_published);
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn send_retained_on_subscribe_test() {
        let topic = unique_id();
        let pub_topic = topic.clone();
        let sub_topic = topic.clone();
        let retain_handling = RetainHandling::SendRetainedOnSubscribe;
        let payload_flag = "111".to_string();
        let client_id = build_client_id("send_retained_on_subscribe_test");

        println!("client_id:{:#?},topic:{:#?}", client_id, sub_topic.clone());

        let addr = broker_addr();
        let sub_topics = &[sub_topic.clone()];
        let sub_qos = &[QOS_0];
        let sub_opts = &[SubscribeOptions::new(true, false, retain_handling)];

        let cli = connect_server5(&client_id, &addr, false, false);

        let message_content = format!("mqtt {payload_flag} message");
        // publish
        let msg = Message::new_retained(pub_topic.clone(), message_content.clone(), QOS_0);
        assert!(cli.publish(msg.clone()).is_ok());

        // subscribe
        let consumer_client_id = build_client_id("send_retained_on_subscribe_test");
        let consumer_cli = connect_server5(&consumer_client_id, &addr, false, false);
        let rx = consumer_cli.start_consuming();
        assert!(consumer_cli
            .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
            .is_ok());

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
        assert!(consumer_cli
            .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
            .is_ok());
        let res = rx.recv_timeout(Duration::from_secs(60));
        assert!(res.is_ok());
        println!("{:?}", res);
        distinct_conn(consumer_cli);
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn send_retained_on_new_test() {
        let topic = unique_id();
        let pub_topic = topic.clone();
        let sub_topic = topic.clone();
        let retain_handling = RetainHandling::SendRetainedOnNew;
        let payload_flag = "111".to_string();

        let client_id = build_client_id("send_retained_on_new_test");

        println!("client_id:{:#?},topic:{:#?}", client_id, sub_topic.clone());

        let addr = broker_addr();
        let sub_topics = &[sub_topic.clone()];
        let sub_qos = &[QOS_0];
        let sub_opts = &[SubscribeOptions::new(true, false, retain_handling)];

        let cli = connect_server5(&client_id, &addr, false, false);

        let message_content = format!("mqtt {payload_flag} message");

        // publish
        let msg = Message::new_retained(pub_topic.clone(), message_content.clone(), QOS_0);
        assert!(cli.publish(msg.clone()).is_ok());

        // new subscribe
        let consumer_client_id = build_client_id("send_retained_on_new_test");
        let consumer_cli = connect_server5(&consumer_client_id, &addr, false, false);
        let rx: paho_mqtt::Receiver<Option<Message>> = consumer_cli.start_consuming();
        assert!(consumer_cli
            .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
            .is_ok());

        recv_retain_msg(message_content.clone(), rx.clone()).await;

        consumer_cli.unsubscribe_many(sub_topics).unwrap();

        distinct_conn(cli);
        distinct_conn(consumer_cli);

        // old subscribe
        let consumer_cli = connect_server5(&consumer_client_id, &addr, false, false);
        let _ = consumer_cli.start_consuming();
        let res = consumer_cli.subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None);
        println!("{:?}", res);
        assert!(res.is_ok());
        distinct_conn(consumer_cli);
    }

    #[tokio::test]
    async fn dont_send_retained_test() {
        let topic = unique_id();
        let pub_topic = topic.clone();
        let sub_topic = topic.clone();
        let retain_handling = RetainHandling::DontSendRetained;
        let payload_flag = "111".to_string();

        let client_id = build_client_id("dont_send_retained_test");

        println!("client_id:{:#?},topic:{:#?}", client_id, sub_topic.clone());

        let addr = broker_addr();
        let sub_topics = &[sub_topic.clone()];
        let sub_qos = &[QOS_0];
        let sub_opts = &[SubscribeOptions::new(true, false, retain_handling)];

        let cli = connect_server5(&client_id, &addr, false, false);

        let message_content = format!("mqtt {payload_flag} message");

        // publish
        let msg = Message::new_retained(pub_topic.clone(), message_content.clone(), QOS_0);
        assert!(cli.publish(msg.clone()).is_ok());

        let consumer_client_id = build_client_id("dont_send_retained_test");

        // new subscribe
        let consumer_cli = connect_server5(&consumer_client_id, &addr, false, false);
        let rx = consumer_cli.start_consuming();
        let res = consumer_cli.subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None);
        println!("{:?}", res);
        assert!(res.is_ok());
        let res = rx.recv_timeout(Duration::from_secs(60));
        println!("{:?}", res);
        assert!(res.is_ok());
        distinct_conn(consumer_cli);
        distinct_conn(cli);
    }

    async fn recv_retain_msg(message_content: String, rx: paho_mqtt::Receiver<Option<Message>>) {
        for msg in rx.iter() {
            let msg = msg.unwrap();
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            println!("retain message:{}", payload.clone());
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
    }
}
