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
    use grpc_clients::{placement::mqtt::call::{placement_create_acl, placement_delete_acl, placement_list_acl}, pool::ClientPool};
    use paho_mqtt::Message;
    use protocol::placement_center::placement_center_mqtt::{CreateAclRequest, DeleteAclRequest, ListAclRequest};

    use crate::mqtt_client::mqtt::common::{broker_addr, broker_grpc_addr, connect_server34, distinct_conn};
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };

    #[tokio::test]
    async fn acl_authorization_test() {
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let topic = "/tests/t1".to_string();

        add_test_acl(client_pool.clone(), grpc_addr.clone(), MqttAclResourceType::ClientId, client_id.clone(), topic.clone(), MqttAclAction::All, MqttAclPermission::Deny).await;
        client_publish_test(&client_id, &addr, &topic).await;
        delete_test_acl(client_pool.clone(), grpc_addr.clone(), MqttAclResourceType::ClientId, client_id.clone(), topic.clone()).await;
    }

    #[tokio::test]
    async fn acl_storage_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = "test_user".to_string();
        let topic = "/tests/t1".to_string();

        add_test_acl(client_pool.clone(), grpc_addr.clone(), MqttAclResourceType::User, username.clone(), topic.clone(), MqttAclAction::All, MqttAclPermission::Deny).await;
 
        let request = ListAclRequest {
            cluster_name: "test".to_string(),
        };

        match placement_list_acl(client_pool.clone(), grpc_addr.clone(), request.clone()).await {
            Ok(res) => {
                let mut flag: bool = false;
                for raw in res.acls {
                    let acl = serde_json::from_slice::<MqttAcl>(raw.as_slice()).unwrap();
                    if acl.resource_type == MqttAclResourceType::User && acl.resource_name == username && acl.topic == topic && acl.action == MqttAclAction::All && acl.permission == MqttAclPermission::Deny {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                assert!(false, "list acl error: {:?}", e);
            }
        }

        delete_test_acl(client_pool.clone(), grpc_addr.clone(), MqttAclResourceType::User, username.clone(), topic.clone()).await;

        match placement_list_acl(client_pool.clone(), grpc_addr.clone(), request.clone()).await {
            Ok(res) => {
                let mut flag: bool = true;
                for raw in res.acls {
                    let acl = serde_json::from_slice::<MqttAcl>(raw.as_slice()).unwrap();
                    if acl.resource_type == MqttAclResourceType::User && acl.resource_name == username && acl.topic == topic && acl.action == MqttAclAction::All && acl.permission == MqttAclPermission::Deny {
                        flag = false;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                assert!(false, "list acl error: {:?}", e);
            }
        }

    }

    #[tokio::test]
    async fn create_acl_test() {
        let username = "test_user".to_string();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let topic = "/tests/t1".to_string();

        add_test_acl(client_pool.clone(), grpc_addr.clone(), MqttAclResourceType::User, username.clone(), topic.clone(), MqttAclAction::All, MqttAclPermission::Deny).await;
    }

    #[tokio::test]
    async fn acl_list_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let request = ListAclRequest {
            cluster_name: "test".to_string(),
        };
        match placement_list_acl(client_pool, grpc_addr, request).await {
            Ok(res) => {
                println!("{:?}", res);
            }
            Err(e) => {
                assert!(false, "list acl error: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn delete_acl_test() {
        let username = "test_user".to_string();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let topic = "/tests/t1".to_string();

        delete_test_acl(client_pool, grpc_addr, MqttAclResourceType::User, username, topic).await;
    }



    async fn add_test_acl(client_pool: Arc<ClientPool>, addr: Vec<String>, resource_type: MqttAclResourceType, resource_name: String, topic: String, action: MqttAclAction, permission: MqttAclPermission) {
        let acl = MqttAcl {
            resource_type,
            resource_name,
            ip: "*".to_string(),
            topic,
            action,
            permission,
        };
        let request = CreateAclRequest {
            cluster_name: "test".to_string(),
            acl: serde_json::to_vec(&acl).unwrap(),
        };
        match placement_create_acl(client_pool.clone(), addr.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                assert!(false, "create acl error: {:?}", e);
            }
        }
    }
    
    async fn delete_test_acl(client_pool: Arc<ClientPool>, addr: Vec<String>, resource_type: MqttAclResourceType, resource_name: String, topic: String) {
        let acl = MqttAcl {
            resource_type,
            resource_name,
            ip: "".to_string(),
            topic: topic.to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Deny,
        };

        let request = DeleteAclRequest {
            cluster_name: "test".to_string(),
            acl: serde_json::to_vec(&acl).unwrap(),
        };
        match placement_delete_acl(client_pool.clone(), addr.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                assert!(false, "delete acl error: {:?}", e);
            }
        }
    }

    async fn client_publish_test(client_id: &str, addr: &str, topic: &str) {
        let mqtt_version = 3;
        let cli = connect_server34(mqtt_version, client_id, addr);

        let msg = Message::new(topic, format!("mqtt message"), 0);
        match cli.publish(msg) {
            Ok(_) => {
                assert!(false, "should not publish success");
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        distinct_conn(cli);
    }


    // async fn client_subscribe_test(client_id: &str, addr: &str, topic: &str) {

    // }
}
