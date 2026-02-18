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
    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, publish_data,
            ssl_by_type, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::uuid::unique_id;
    use paho_mqtt::{MessageBuilder, Properties, PropertyCode, SubscribeOptions};
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn sub_identifier_test() {
        let network = "tcp";
        let qos = 1;
        let client_id =
            build_client_id(format!("sub_identifier_test_pub_{network}_{qos}").as_str());

        let topic = unique_id();
        let topic1 = format!("/sub_identifier_test/{topic}/+");
        let topic2 = format!("/sub_identifier_test/{topic}/test");
        let topic3 = format!("/sub_identifier_test/{topic}/test_one");

        // publish
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };

        let cli = connect_server(&client_properties);
        let message_content = "sub_identifier_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .topic(topic2.clone())
            .payload(message_content.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        // subscribe
        let client_id =
            build_client_id(format!("sub_identifier_test_sub_{network}_{qos}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };

        // sub1
        let cli = connect_server(&client_properties);
        let mut props: Properties = Properties::new();
        props
            .push_int(PropertyCode::SubscriptionIdentifier, 1)
            .unwrap();
        let res = cli.subscribe_many_with_options(
            std::slice::from_ref(&topic1),
            &[qos],
            &[SubscribeOptions::default()],
            Some(props),
        );
        assert!(res.is_ok());

        // sub2
        let mut props: Properties = Properties::new();
        props
            .push_int(PropertyCode::SubscriptionIdentifier, 2)
            .unwrap();

        let res = cli.subscribe_many_with_options(
            std::slice::from_ref(&topic2),
            &[qos],
            &[SubscribeOptions::default()],
            Some(props),
        );
        assert!(res.is_ok());

        // sub data
        let mut r_one = false;
        let mut r_two = false;
        let rx = cli.start_consuming();

        let timeout = Duration::from_secs(60);
        let poll_interval = Duration::from_secs(1);
        let start = Instant::now();
        loop {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                panic!(
                    "Timeout waiting for messages with sub_identifier after {} seconds (r_one={r_one}, r_two={r_two})",
                    timeout.as_secs()
                );
            }
            let remaining = timeout.saturating_sub(elapsed);
            let wait_for = remaining.min(poll_interval);
            let res_opt = rx.recv_timeout(wait_for);
            match res_opt {
                Ok(Some(msg)) => {
                    println!("message: {msg:?}");
                    if let Some(id) = msg
                        .properties()
                        .get_int(PropertyCode::SubscriptionIdentifier)
                    {
                        println!("sub_identifier: {id}");
                        match id {
                            1 => r_one = true,
                            2 => r_two = true,
                            _ => panic!("unexpected sub_identifier: {}", id),
                        }
                    }
                }
                Ok(None) => continue,
                Err(_) => continue,
            }
            println!("r_one: {r_one}, r_two: {r_two}");
            if r_one && r_two {
                break;
            }
        }

        // publish data
        let msg = MessageBuilder::new()
            .topic(topic3.clone())
            .payload(message_content.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        let timeout2 = Duration::from_secs(60);
        let poll_interval2 = Duration::from_secs(1);
        let start2 = Instant::now();
        loop {
            let elapsed = start2.elapsed();
            if elapsed >= timeout2 {
                panic!(
                    "Timeout waiting for message on topic3 after {} seconds",
                    timeout2.as_secs()
                );
            }
            let remaining = timeout2.saturating_sub(elapsed);
            let wait_for = remaining.min(poll_interval2);
            let res_opt = rx.recv_timeout(wait_for);
            match res_opt {
                Ok(Some(msg)) => {
                    if let Some(id) = msg
                        .properties()
                        .get_int(PropertyCode::SubscriptionIdentifier)
                    {
                        if id == 1 {
                            break;
                        }
                    }
                }
                Ok(None) => continue,
                Err(_) => continue,
            }
        }

        distinct_conn(cli);
    }
}
