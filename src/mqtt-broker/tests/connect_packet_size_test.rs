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
    use crate::common::{broker_addr, broker_ssl_addr, connect_server5_packet_size, distinct_conn};
    use common_base::tools::unique_id;
    use paho_mqtt::{MessageBuilder, QOS_1};

    #[tokio::test]
    async fn client5_packet_size_test_tcp() {
        let topic = unique_id();
        let topic = format!("/packet_tcp/{}", topic);
        let addr = broker_addr();
        simple_test(addr, topic, 0, 128, false).await;
    }

    #[tokio::test]
    async fn client5_packet_size_test_tcp_ssl() {
        let topic = unique_id();
        let topic = format!("/packet_tcp_ssl/{}", topic);
        let addr = broker_ssl_addr();
        simple_test(addr, topic, 0, 128, true).await;
    }

    async fn simple_test(addr: String, topic: String, sub_qos: i32, packet_size: i32, ssl: bool) {
        let client_id = unique_id();
        let cli = connect_server5_packet_size(&client_id, &addr, packet_size, false, ssl);

        let message_content = vec!['a'; (packet_size + 1) as usize].iter().collect::<String>();
        // publish
        let msg = MessageBuilder::new()
            .topic(topic.clone())
            .payload(message_content.clone())
            .qos(QOS_1)
            .finalize();

        match cli.publish(msg) {
            Ok(_) => {
                assert!(false);
            }
            Err(e) => {
                println!("{}",e.to_string());
                assert!(true);
            }
        }

        let message_content = vec!['a'; packet_size as usize].iter().collect::<String>();
        // publish
        let msg = MessageBuilder::new()
            .topic(topic.clone())
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
        match cli.subscribe(topic.as_str(), sub_qos) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        for msg in rx.iter() {
            if let Some(msg) = msg {
                let payload = String::from_utf8(msg.payload().to_vec()).unwrap();

                println!("len: {} {}", payload.len(), payload);

                assert!(payload.len() <= packet_size as usize);
                assert_eq!(message_content, payload);

                break;
            } else {
                assert!(false);
            }
        }

        distinct_conn(cli);
    }
}
