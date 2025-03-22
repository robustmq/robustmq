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
    use std::process;
    use std::time::Duration;

    use common_base::tools::{now_second, unique_id};
    use paho_mqtt::{
        Client, ConnectOptionsBuilder, MessageBuilder, Properties, PropertyCode, QOS_1,
    };

    use crate::mqtt_protocol::common::{
        broker_addr, build_client_id, build_create_pros, build_v5_pros, connect_server5,
        distinct_conn, password, username,
    };

    #[tokio::test]
    async fn last_will_message_test() {
        let client_id = build_client_id("last_will_message_test");
        let addr = broker_addr();

        // create connection
        let create_opts = build_create_pros(&client_id, &addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        // build will message
        let mut props = Properties::new();
        props.push_u32(PropertyCode::WillDelayInterval, 2).unwrap();
        let will_message_content = "will message content".to_string();
        let will_topic = format!("/tests/{}", unique_id());
        let will = MessageBuilder::new()
            .properties(props)
            .payload(will_message_content.clone())
            .topic(will_topic.clone())
            .qos(QOS_1)
            .retained(false)
            .finalize();

        // create connection
        let create_props = build_v5_pros();
        let conn_opts = ConnectOptionsBuilder::new_v5()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_start(true)
            .connect_timeout(Duration::from_secs(5))
            .properties(create_props.clone())
            .will_message(will)
            .user_name(username())
            .password(password())
            .finalize();

        match cli.connect(conn_opts) {
            Ok(_) => {}
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);

        // sub will message topic
        let start = now_second();
        let sub_topics = &[will_topic.clone()];
        let client_id = build_client_id("last_will_message_test");
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
            println!("recv message: {}", payload);
            if payload == will_message_content {
                println!("{}", now_second() - start);
                break;
            }
        }
        distinct_conn(cli);
    }
}
