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
    use crate::mqtt_protocol::common::{
        broker_addr_by_type, broker_grpc_addr, build_client_id, connect_server, distinct_conn,
        network_types, protocol_versions, qos_list, ssl_by_type, ws_by_type,
    };
    use crate::mqtt_protocol::ClientTestProperties;
    use common_base::tools::unique_id;
    use grpc_clients::mqtt::admin::call::{
        mqtt_broker_create_acl, mqtt_broker_create_user, mqtt_broker_delete_acl,
        mqtt_broker_delete_user, mqtt_broker_list_acl,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };
    use paho_mqtt::MessageBuilder;
    use protocol::broker_mqtt::broker_mqtt_admin::{
        CreateAclRequest, CreateUserRequest, DeleteAclRequest, DeleteUserRequest, ListAclRequest,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn acl_storage_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];
        let cluster_name: String = format!("test_cluster_{}", unique_id());

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: "acl_storage_test".to_string(),
            topic: "tp-1".to_string(),
            ip: "*".to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        };

        create_acl(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            acl.clone(),
        )
        .await;

        let list_request = ListAclRequest {
            cluster_name: cluster_name.clone(),
        };
        match mqtt_broker_list_acl(&client_pool, &grpc_addr, list_request.clone()).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.acls {
                    let tmp = serde_json::from_slice::<MqttAcl>(raw.as_slice()).unwrap();
                    if tmp.resource_type == acl.resource_type
                        && tmp.resource_name == acl.resource_name
                        && tmp.topic == acl.topic
                        && tmp.ip == acl.ip
                        && tmp.action == acl.action
                        && tmp.permission == acl.permission
                    {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                panic!("list acl error: {:?}", e);
            }
        };

        delete_acl(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            acl.clone(),
        )
        .await;

        match mqtt_broker_list_acl(&client_pool, &grpc_addr, list_request.clone()).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.acls {
                    let tmp = serde_json::from_slice::<MqttAcl>(raw.as_slice()).unwrap();
                    if tmp.resource_type == acl.resource_type
                        && tmp.resource_name == acl.resource_name
                        && tmp.topic == acl.topic
                        && tmp.ip == acl.ip
                        && tmp.action == acl.action
                        && tmp.permission == acl.permission
                    {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                panic!("list acl error: {:?}", e);
            }
        };
    }

    #[tokio::test]
    async fn user_publish_authorization_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let cluster_name: String = unique_id();

        let topic = format!(
            "{}/{}",
            "/tests/user_publish_authorization_test",
            unique_id()
        );

        let username = unique_id();
        let password = "caclpublic".to_string();

        create_user(
            client_pool.clone(),
            grpc_addr.clone(),
            username.clone(),
            password.clone(),
        )
        .await;

        publish_user_acl_test(&topic, username.clone(), password.clone(), false).await;

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: username.clone(),
            topic: topic.clone(),
            ip: "*".to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        };

        create_acl(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            acl.clone(),
        )
        .await;

        publish_user_acl_test(&topic, username.clone(), password.clone(), true).await;

        delete_acl(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            acl.clone(),
        )
        .await;

        publish_user_acl_test(&topic, username.clone(), password.clone(), false).await;
        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
    }

    #[tokio::test]
    async fn client_publish_authorization_test() {
        let client_id = build_client_id("client_publish_authorization_test");

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = unique_id();
        let password = "caclpublic".to_string();

        create_user(
            client_pool.clone(),
            grpc_addr.clone(),
            username.clone(),
            password.clone(),
        )
        .await;

        let cluster_name: String = unique_id();

        let topic = format!(
            "{}/{}",
            "/tests/client_publish_authorization_test",
            unique_id()
        );

        publish_client_id_acl_test(&topic, &client_id, &username, &password, false).await;

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: client_id.clone(),
            topic: topic.clone(),
            ip: "*".to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        };

        create_acl(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            acl.clone(),
        )
        .await;

        publish_client_id_acl_test(&topic, &client_id, &username, &password, true).await;

        delete_acl(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            acl.clone(),
        )
        .await;

        publish_client_id_acl_test(&topic, &client_id, &username, &password, false).await;

        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
    }

    async fn publish_user_acl_test(topic: &str, username: String, password: String, is_err: bool) {
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

    async fn publish_client_id_acl_test(
        topic: &str,
        client_id: &str,
        username: &str,
        password: &str,
        is_err: bool,
    ) {
        let protocol = 5;
        let network = "tcp";
        let qos = 1;

        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_owned(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            user_name: username.to_owned(),
            password: password.to_owned(),
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

        if is_err {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }

        distinct_conn(cli);
    }

    async fn create_user(
        client_pool: Arc<ClientPool>,
        grpc_addr: Vec<String>,
        username: String,
        password: String,
    ) {
        let user = CreateUserRequest {
            username,
            password,
            is_superuser: false,
        };
        let res = mqtt_broker_create_user(&client_pool, &grpc_addr, user.clone()).await;
        assert!(res.is_ok());
    }

    async fn delete_user(client_pool: Arc<ClientPool>, grpc_addr: Vec<String>, username: String) {
        let user = DeleteUserRequest { username };
        let res = mqtt_broker_delete_user(&client_pool, &grpc_addr, user.clone()).await;
        assert!(res.is_ok());
    }

    async fn create_acl(
        client_pool: Arc<ClientPool>,
        grpc_addr: Vec<String>,
        cluster_name: String,
        acl: MqttAcl,
    ) {
        let create_request = CreateAclRequest {
            cluster_name,
            acl: acl.encode().unwrap(),
        };

        let res = mqtt_broker_create_acl(&client_pool, &grpc_addr, create_request).await;
        assert!(res.is_ok());
    }

    async fn delete_acl(
        client_pool: Arc<ClientPool>,
        grpc_addr: Vec<String>,
        cluster_name: String,
        acl: MqttAcl,
    ) {
        let delete_request = DeleteAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };

        let res = mqtt_broker_delete_acl(&client_pool, &grpc_addr, delete_request).await;
        assert!(res.is_ok());
    }
}
