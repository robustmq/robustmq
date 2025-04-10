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
    use crate::mqtt_protocol::{
        common::{
            broker_addr_by_type, broker_grpc_addr, build_client_id, connect_server, distinct_conn,
            publish_data, ssl_by_type, subscribe_data_by_qos, ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::tools::unique_id;
    use grpc_clients::mqtt::admin::call::mqtt_broker_create_topic_rewrite_rule;
    use grpc_clients::pool::ClientPool;
    use paho_mqtt::{Message, MessageBuilder};
    use protocol::broker_mqtt::broker_mqtt_admin::CreateTopicRewriteRuleRequest;
    use std::sync::Arc;

    #[tokio::test]
    async fn publish_rewrite_test() {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let action: String = "All".to_string();

        let prefix = format!("{}{}", "topic_rewrite_rule_test", unique_id());
        let client_id = build_client_id(&prefix);

        let req = CreateTopicRewriteRuleRequest {
            action: action.clone(),
            source_topic: format!("{}y/+/z/#", prefix),
            dest_topic: format!("{}y/z/$2", prefix),
            regex: format!("^{}y/(.+)/z/(.+)$", prefix),
        };
        let res = mqtt_broker_create_topic_rewrite_rule(&client_pool, &grpc_addr, req).await;
        assert!(res.is_ok());

        let source_topic = format!("{}y/a/z/b", prefix);
        let rewrite_topic = format!("{}y/z/b", prefix);

        let network = "tcp";
        let protocol = 5;
        let qos = 1;

        // publish
        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let message = "mqtt message".to_string();
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

        // unscriber
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
