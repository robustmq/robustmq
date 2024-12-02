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
        mqtt_broker_create_acl, mqtt_broker_create_user, mqtt_broker_delete_acl,
        mqtt_broker_delete_user, mqtt_broker_list_acl,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };
    use paho_mqtt::{Client, Message};
    use protocol::broker_mqtt::broker_mqtt_admin::{
        CreateAclRequest, CreateUserRequest, DeleteAclRequest, DeleteUserRequest, ListAclRequest,
    };

    use crate::mqtt_client::common::{
        broker_addr, broker_grpc_addr, connect_server5_by_user_information, distinct_conn,
    };

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
        match mqtt_broker_list_acl(client_pool.clone(), &grpc_addr, list_request.clone()).await {
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

        match mqtt_broker_list_acl(client_pool.clone(), &grpc_addr, list_request.clone()).await {
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
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let cluster_name: String = "mqtt-broker".to_string();

        let topic = "/tests/t1".to_string();

        let username = "client_acl".to_string();
        let password = "caclpublic".to_string();

        create_user(
            client_pool.clone(),
            grpc_addr.clone(),
            username.clone(),
            password.clone(),
        )
        .await;

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

        publish_deny_test(
            &client_id,
            &addr,
            &topic,
            username.clone(),
            password.clone(),
        )
        .await;

        delete_acl(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            acl.clone(),
        )
        .await;
        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
    }

    #[tokio::test]
    async fn client_publish_authorization_test() {
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let cluster_name: String = "mqtt-broker".to_string();

        let topic = "/tests/t1".to_string();

        let username = "client_acl".to_string();
        let password = "caclpublic".to_string();

        create_user(
            client_pool.clone(),
            grpc_addr.clone(),
            username.clone(),
            password.clone(),
        )
        .await;

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

        publish_deny_test(
            &client_id,
            &addr,
            &topic,
            username.clone(),
            password.clone(),
        )
        .await;

        delete_acl(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            acl.clone(),
        )
        .await;
        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
    }

    async fn publish_deny_test(
        client_id: &str,
        addr: &str,
        topic: &str,
        username: String,
        password: String,
    ) {
        let cli: Client =
            connect_server5_by_user_information(client_id, addr, username, password, false, false);

        let msg = Message::new(topic, "mqtt message test".to_string(), 1);
        match cli.publish(msg) {
            Ok(_) => {
                panic!("should not publish success");
            }
            Err(e) => {
                println!("{:?}", e);
            }
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
        match mqtt_broker_create_user(client_pool.clone(), &grpc_addr, user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    async fn delete_user(client_pool: Arc<ClientPool>, grpc_addr: Vec<String>, username: String) {
        let user = DeleteUserRequest { username };
        match mqtt_broker_delete_user(client_pool.clone(), &grpc_addr, user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
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

        mqtt_broker_create_acl(client_pool, &grpc_addr, create_request)
            .await
            .unwrap();
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

        mqtt_broker_delete_acl(client_pool, &grpc_addr, delete_request)
            .await
            .unwrap();
    }
}
