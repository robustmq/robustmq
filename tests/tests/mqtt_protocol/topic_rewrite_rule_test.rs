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
            network_types, protocol_versions, publish_data, qos_list, ssl_by_type,
            subscribe_data_by_qos, ws_by_type,
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
    async fn topic_rewrite_rule_test() {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let action: String = "All".to_string();

        for protocol in protocol_versions() {
            for network in network_types() {
                for qos in qos_list() {
                    let client_id = build_client_id(
                        format!("topic_rewrite_rule_test_{}_{}", network, qos).as_str(),
                    );

                    let prefix = format!("/{}/{}_{}", network, qos, protocol);

                    let source_topic: String = format!("{}{}", prefix, "/tests_r1/#");
                    let dest_topic: String = format!("{}{}", prefix, "/test_rewrite/$1");
                    let re: String = format!("{}{}", prefix, "^/tests_r1/(.+)$");

                    let req = CreateTopicRewriteRuleRequest {
                        action: action.clone(),
                        source_topic: source_topic.clone(),
                        dest_topic: dest_topic.clone(),
                        regex: re.clone(),
                    };
                    let res =
                        mqtt_broker_create_topic_rewrite_rule(&client_pool, &grpc_addr, req).await;
                    assert!(res.is_ok());

                    let uuid = unique_id();
                    let topic = format!("{}/tests_r1/{}", prefix, uuid);
                    let rewrite_topic = format!("{}/test_rewrite/{}", prefix, uuid);

                    // publish
                    let client_properties = ClientTestProperties {
                        mqtt_version: protocol,
                        client_id: client_id.to_string(),
                        addr: broker_addr_by_type(&network),
                        ws: ws_by_type(&network),
                        ssl: ssl_by_type(&network),
                        ..Default::default()
                    };
                    let cli = connect_server(&client_properties);
                    let message = "mqtt message".to_string();
                    let msg = MessageBuilder::new()
                        .payload(message.clone())
                        .topic(topic.clone())
                        .qos(qos)
                        .finalize();
                    publish_data(&cli, msg, false);

                    // sub data
                    let call_fn = |msg: Message| {
                        let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                        if payload == message {
                            return true;
                        }
                        false
                    };
                    subscribe_data_by_qos(&cli, &rewrite_topic, qos, call_fn);
                    distinct_conn(cli);
                }
            }
        }
    }
}
