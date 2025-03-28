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
    use std::sync::Arc;

    use common_base::tools::unique_id;
    use grpc_clients::mqtt::admin::call::{
        mqtt_broker_create_blacklist, mqtt_broker_delete_blacklist, mqtt_broker_list_blacklist,
    };
    use grpc_clients::pool::ClientPool;

    use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
    use paho_mqtt::MessageBuilder;
    use protocol::broker_mqtt::broker_mqtt_admin::{
        CreateBlacklistRequest, DeleteBlacklistRequest, ListBlacklistRequest,
    };

    use crate::mqtt_protocol::common::{
        broker_addr_by_type, broker_grpc_addr, build_client_id, connect_server, distinct_conn,
        network_types, protocol_versions, qos_list, ssl_by_type, ws_by_type,
    };
    use crate::mqtt_protocol::ClientTestProperties;

    #[tokio::test]
    async fn blacklist_storage_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let cluster_name: String = format!("test_cluster_{}", unique_id());

        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: "acl_storage_test".to_string(),
            end_time: 10000,
            desc: "".to_string(),
        };

        create_blacklist(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            blacklist.clone(),
        )
        .await;

        let list_request = ListBlacklistRequest {
            cluster_name: cluster_name.clone(),
        };
        match mqtt_broker_list_blacklist(&client_pool, &grpc_addr, list_request.clone()).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.blacklists {
                    let tmp = serde_json::from_slice::<MqttAclBlackList>(raw.as_slice()).unwrap();
                    if tmp.blacklist_type == blacklist.blacklist_type
                        && tmp.resource_name == blacklist.resource_name
                        && tmp.end_time == blacklist.end_time
                        && tmp.desc == blacklist.desc
                    {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                panic!("list blacklist error: {:?}", e);
            }
        };

        delete_blacklist(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            blacklist.clone(),
        )
        .await;

        match mqtt_broker_list_blacklist(&client_pool, &grpc_addr, list_request.clone()).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.blacklists {
                    let tmp = serde_json::from_slice::<MqttAclBlackList>(raw.as_slice()).unwrap();
                    if tmp.blacklist_type == blacklist.blacklist_type
                        && tmp.resource_name == blacklist.resource_name
                        && tmp.end_time == blacklist.end_time
                        && tmp.desc == blacklist.desc
                    {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                panic!("list blacklist error: {:?}", e);
            }
        };
    }

    async fn _publish_deny_test(topic: &str, username: String, password: String, is_err: bool) {
        for protocol in protocol_versions() {
            for network in network_types() {
                for qos in qos_list() {
                    let client_id = build_client_id(
                        format!("publish_qos_test_{}_{}_{}", protocol, network, qos).as_str(),
                    );

                    let client_properties = ClientTestProperties {
                        mqtt_version: protocol,
                        client_id: client_id.to_string(),
                        addr: broker_addr_by_type(&network),
                        ws: ws_by_type(&network),
                        ssl: ssl_by_type(&network),
                        user_name: username.clone(),
                        password: password.clone(),
                        ..Default::default()
                    };
                    let cli = connect_server(&client_properties);

                    // publish retain
                    let message = "mqtt message".to_string();
                    let msg = MessageBuilder::new()
                        .payload(message.clone())
                        .topic(topic.to_owned())
                        .qos(qos)
                        .retained(true)
                        .finalize();

                    let result = cli.publish(msg);

                    if result.is_err() {
                        println!("{},{:?},{}", client_id, result, is_err);
                    }

                    if protocol == 5 && is_err && qos != 0 {
                        assert!(result.is_err());
                    } else {
                        assert!(result.is_ok());
                    }

                    distinct_conn(cli);
                }
            }
        }
    }

    async fn create_blacklist(
        client_pool: Arc<ClientPool>,
        grpc_addr: Vec<String>,
        cluster_name: String,
        blacklist: MqttAclBlackList,
    ) {
        let create_request = CreateBlacklistRequest {
            cluster_name,
            blacklist: blacklist.encode().unwrap(),
        };

        let res = mqtt_broker_create_blacklist(&client_pool, &grpc_addr, create_request).await;
        assert!(res.is_ok());
    }

    async fn delete_blacklist(
        client_pool: Arc<ClientPool>,
        grpc_addr: Vec<String>,
        cluster_name: String,
        blacklist: MqttAclBlackList,
    ) {
        let delete_request = DeleteBlacklistRequest {
            cluster_name,
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name,
        };

        let res = mqtt_broker_delete_blacklist(&client_pool, &grpc_addr, delete_request).await;
        assert!(res.is_ok());
    }
}
