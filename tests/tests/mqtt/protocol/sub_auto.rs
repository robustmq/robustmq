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
            broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
            publish_data, ssl_by_type, ws_by_type,
        },
        ClientTestProperties,
    };
    use admin_server::client::AdminHttpClient;
    use admin_server::mqtt::subscribe::{CreateAutoSubscribeReq, DeleteAutoSubscribeReq};
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, QOS_1};
    use std::time::Duration;

    #[tokio::test]
    async fn sub_auto_test() {
        let admin_client = create_test_env().await;

        let uniq = unique_id();
        let topic = format!("/sub_auto_test/v1/v2/{uniq}");

        let network = "tcp";
        let qos = 2;

        // create_auto_subscribe_rule
        create_auto_subscribe_rule(&admin_client, &topic).await;

        // publish
        let client_id = build_client_id(format!("sub_auto_test_{network}_{qos}").as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let message_content = "sub_auto_test mqtt message".to_string();
        let msg = Message::new(topic.clone(), message_content.clone(), QOS_1);
        publish_data(&cli, msg, false);

        // auto consuming
        let rx = cli.start_consuming();
        loop {
            let res = rx.recv_timeout(Duration::from_secs(10));
            println!("{res:?}");
            if let Ok(msg_opt) = res {
                assert!(msg_opt.is_some());
                let msg = msg_opt.unwrap();
                println!("{msg:?}");
                break;
            }
        }
        distinct_conn(cli);

        // delete_auto_subscribe_rule
        delete_auto_subscribe_rule(&admin_client, &topic).await;
    }

    async fn create_auto_subscribe_rule(admin_client: &AdminHttpClient, topic: &str) {
        let request = CreateAutoSubscribeReq {
            topic: topic.to_string(),
            qos: 1,
            no_local: false,
            retain_as_published: false,
            retained_handling: 0,
        };
        let res = admin_client.create_auto_subscribe(&request).await;
        assert!(res.is_ok());
    }

    async fn delete_auto_subscribe_rule(admin_client: &AdminHttpClient, topic: &str) {
        let request = DeleteAutoSubscribeReq {
            topic_name: topic.to_string(),
        };
        let res = admin_client.delete_auto_subscribe(&request).await;
        assert!(res.is_ok());
    }
}
