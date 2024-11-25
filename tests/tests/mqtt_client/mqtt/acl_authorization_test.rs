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
    use grpc_clients::{mqtt::admin::call::{mqtt_broker_create_acl, mqtt_broker_delete_acl, mqtt_broker_list_acl}, pool::ClientPool};
    use paho_mqtt::Message;
    use protocol::broker_mqtt::broker_mqtt_admin::{CreateAclRequest, DeleteAclRequest, ListAclRequest};

    use crate::mqtt_client::mqtt::common::{broker_addr, broker_grpc_addr, connect_server34, distinct_conn};
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };

    #[tokio::test]
    async fn acl_publish_authorization_test() {
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let cluster_name: String = format!("test_cluster_{}", unique_id());

        let topic = "test-1".to_string();

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: client_id.clone(),
            topic: topic.clone(),
            ip: "*".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Deny,
        };

        let create_request = CreateAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };

        mqtt_broker_create_acl(client_pool.clone(), grpc_addr.clone(), create_request).await.unwrap();

        client_publish_test(&client_id, &addr, &topic).await;

        let delete_request = DeleteAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };

        mqtt_broker_delete_acl(client_pool.clone(), grpc_addr.clone(), delete_request).await.unwrap();
    }

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
            action: MqttAclAction::All,
            permission: MqttAclPermission::Deny,
        };

        let create_request = CreateAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };

        mqtt_broker_create_acl(client_pool.clone(), grpc_addr.clone(), create_request).await.unwrap();

        let list_request = ListAclRequest {
            cluster_name: cluster_name.clone(),
        };

        match mqtt_broker_list_acl(client_pool.clone(), grpc_addr.clone(), list_request.clone()).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.acls{
                    let tmp = serde_json::from_slice::<MqttAcl>(raw.as_slice()).unwrap();
                    if tmp.resource_type == acl.resource_type && tmp.resource_name == acl.resource_name && tmp.topic == acl.topic && tmp.ip == acl.ip && tmp.action == acl.action && tmp.permission == acl.permission {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                assert!(false, "list acl error: {:?}", e);
            }
        };

        let delete_request = DeleteAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };

        match mqtt_broker_delete_acl(client_pool.clone(), grpc_addr.clone(), delete_request).await {
            Ok(_) => {}
            Err(e) => {
                assert!(false, "delete acl error: {:?}", e);
            }
        };

        match mqtt_broker_list_acl(client_pool.clone(), grpc_addr.clone(), list_request.clone()).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.acls{
                    let tmp = serde_json::from_slice::<MqttAcl>(raw.as_slice()).unwrap();
                    if tmp.resource_type == acl.resource_type && tmp.resource_name == acl.resource_name && tmp.topic == acl.topic && tmp.ip == acl.ip && tmp.action == acl.action && tmp.permission == acl.permission {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                assert!(false, "list acl error: {:?}", e);
            }
        };

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

}
