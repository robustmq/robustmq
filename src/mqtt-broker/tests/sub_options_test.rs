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
    use std::time::Duration;

    use common_base::tools::unique_id;
    use paho_mqtt::{Message, RetainHandling, SubscribeOptions, QOS_0, QOS_2};

    use crate::common::{broker_addr, connect_server5, distinct_conn};

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

    #[tokio::test]
    async fn sub_options_retain_handling_test() {
        let mut i = 0;
        for retain in vec![
            RetainHandling::SendRetainedOnSubscribe,
            RetainHandling::SendRetainedOnNew,
            RetainHandling::DontSendRetained,
        ] {
            let topic = format!("/tests/{}", unique_id());
            retain_handling_test(topic.clone(), topic.clone(), retain, i.to_string()).await;
            i += 1;
        }
    }

    async fn no_local_test(
        pub_topic: String,
        sub_topic: String,
        no_local: bool,
        payload_flag: String,
    ) {
        let client_id = unique_id();
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

        match rx.recv_timeout(Duration::from_secs(1)) {
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
        let client_id = unique_id();
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
        let recv = rx.recv_timeout(Duration::from_secs(1));
        assert!(recv.is_ok());
        assert_eq!(recv.unwrap().unwrap().retained(), retain_as_published);
        distinct_conn(cli);
    }

    async fn retain_handling_test(
        pub_topic: String,
        sub_topic: String,
        retain_handling: RetainHandling,
        payload_flag: String,
    ) {
        let client_id = unique_id();

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
        let consumer_client_id = unique_id();
        let consumer_cli = connect_server5(&consumer_client_id, &addr, false, false);
        let rx = consumer_cli.start_consuming();
        assert!(consumer_cli
            .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
            .is_ok());

        match retain_handling {
            RetainHandling::SendRetainedOnSubscribe => {
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
                let _ = consumer_cli.unsubscribe_many(sub_topics);
                assert!(consumer_cli
                    .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
                    .is_ok());
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_err());
            }
            RetainHandling::SendRetainedOnNew => {
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
                let _ = consumer_cli.unsubscribe_many(sub_topics);
                assert!(consumer_cli
                    .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
                    .is_ok());
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_err());
            }
            RetainHandling::DontSendRetained => {
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
                assert!(rx.recv_timeout(Duration::from_secs(1)).is_err());
            }
        }
        distinct_conn(consumer_cli);
        distinct_conn(cli);
    }
}
