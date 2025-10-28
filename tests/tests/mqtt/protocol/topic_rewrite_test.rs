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

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
            publish_data, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };
    use admin_server::mqtt::topic::CreateTopicRewriteReq;
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, MessageBuilder};
    use tokio::time::sleep;

    #[tokio::test]
    async fn pub_sub_rewrite_test() {
        let admin_client = create_test_env().await;

        let action: String = "All".to_string();

        let prefix = format!("{}{}", "pub_sub_rewrite_test", unique_id());
        let client_id = build_client_id(&prefix);

        let req = CreateTopicRewriteReq {
            action: action.clone(),
            source_topic: format!("{prefix}y/+/z/#"),
            dest_topic: format!("{prefix}y/z/$2"),
            regex: format!("^{prefix}y/(.+)/z/(.+)$"),
        };
        println!("{:?}", req);
        let res = admin_client.create_topic_rewrite(&req).await;
        assert!(res.is_ok());

        sleep(Duration::from_secs(30)).await;
        let source_topic = format!("{prefix}y/a/z/b");
        let rewrite_topic = format!("{prefix}y/z/b");

        let network = "tcp";
        let protocol = 5;
        let qos = 1;

        // publish 1
        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message = "pub_sub_rewrite_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(source_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        sleep(Duration::from_secs(30)).await;

        // publish 2
        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message = "pub_sub_rewrite_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(source_topic.clone())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);

        // sub data by rewrite_topic
        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            if payload == message {
                return true;
            }
            false
        };
        subscribe_data_by_qos(&cli, &rewrite_topic, qos, call_fn);
        distinct_conn(cli);

        // sub data by rewrite_topic
        let prefix = format!("{}{}", "pub_sub_rewrite_test", unique_id());
        let client_id = build_client_id(&prefix);
        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        // sub data by rewrite_topic
        let call_fn = |msg: Message| {
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            if payload == message {
                return true;
            }
            false
        };
        subscribe_data_by_qos(&cli, &source_topic, qos, call_fn);
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn un_sub_rewrite_test() {
        let admin_client = create_test_env().await;

        let action: String = "All".to_string();

        let prefix = format!("{}{}", "pub_sub_rewrite_test", unique_id());

        let req = CreateTopicRewriteReq {
            action: action.clone(),
            source_topic: format!("{prefix}y/+/z/#"),
            dest_topic: format!("{prefix}y/z/$2"),
            regex: format!("^{prefix}y/(.+)/z/(.+)$"),
        };
        let res = admin_client.create_topic_rewrite(&req).await;
        assert!(res.is_ok());

        sleep(Duration::from_secs(10)).await;
        let source_topic = format!("{prefix}y/a/z/b");

        let network = "tcp";
        let protocol = 5;
        let qos = 1;

        let prefix = format!("{}{}", "topic_rewrite_rule_test", unique_id());
        let client_id = build_client_id(&prefix);
        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let res = cli.subscribe(&source_topic, qos);
        assert!(res.is_ok());
        let res = cli.unsubscribe(&source_topic);
        assert!(res.is_ok());
        distinct_conn(cli);
    }
}
