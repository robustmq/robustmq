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
    use std::{sync::Arc, time::Duration};

    use common_base::tools::unique_id;
    use grpc_clients::{
        mqtt::admin::call::{
            mqtt_broker_delete_auto_subscribe_rule, mqtt_broker_set_auto_subscribe_rule,
        },
        pool::ClientPool,
    };
    use paho_mqtt::{Message, QOS_1};
    use protocol::broker_mqtt::broker_mqtt_admin::{
        DeleteAutoSubscribeRuleRequest, SetAutoSubscribeRuleRequest,
    };

    use crate::mqtt_protocol::{
        common::{
            broker_addr_by_type, broker_grpc_addr, build_client_id, connect_server, distinct_conn,
            publish_data, ssl_by_type, ws_by_type,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn sub_auto_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let uniq = unique_id();
        let topic = format!("/tests/v1/v2/{}", uniq);

        let network = "tcp";
        let qos = 2;

        // create_auto_subscribe_rule
        create_auto_subscribe_rule(&client_pool, grpc_addr.clone(), &topic).await;

        // publish
        let client_id = build_client_id(format!("sub_auto_test_{}_{}", network, qos).as_str());

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
            println!("{:?}", res);
            if let Ok(msg_opt) = res {
                assert!(msg_opt.is_some());
                let msg = msg_opt.unwrap();
                println!("{:?}", msg);
                break;
            }
        }
        distinct_conn(cli);

        // delete_auto_subscribe_rule
        delete_auto_subscribe_rule(&client_pool, grpc_addr.clone(), &topic).await;
    }

    async fn create_auto_subscribe_rule(
        client_pool: &Arc<ClientPool>,
        grpc_addr: Vec<String>,
        topic: &str,
    ) {
        let request = SetAutoSubscribeRuleRequest {
            topic: topic.to_string(),
            qos: 1,
            no_local: false,
            retain_as_published: false,
            retained_handling: 0,
        };
        let res = mqtt_broker_set_auto_subscribe_rule(client_pool, &grpc_addr, request).await;
        assert!(res.is_ok());
    }

    async fn delete_auto_subscribe_rule(
        client_pool: &Arc<ClientPool>,
        grpc_addr: Vec<String>,
        topic: &str,
    ) {
        let request = DeleteAutoSubscribeRuleRequest {
            topic: topic.to_string(),
        };
        let res = mqtt_broker_delete_auto_subscribe_rule(client_pool, &grpc_addr, request).await;
        assert!(res.is_ok());
    }
}
