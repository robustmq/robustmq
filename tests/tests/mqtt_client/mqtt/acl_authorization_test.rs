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
    use grpc_clients::{placement::mqtt::call::{create_acl, delete_acl}, pool::ClientPool};
    use paho_mqtt::Message;

    use crate::mqtt_client::mqtt::common::{broker_addr, broker_grpc_addr, connect_server34};
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
        publish_test(&client_id, &addr, &topic).await;
        delete_test_acl(client_pool.clone(), grpc_addr.clone(), MqttAclResourceType::ClientId, client_id.clone(), topic.clone()).await;
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
            acl,
        };
        create_acl(client_pool.clone(), addr.clone(), request).await;
    }
    
    async fn delete_test_acl(client_pool: Arc<ClientPool>, addr: Vec<String>, resource_type: MqttAclResourceType, resource_name: String, topic: String) {
        let acl = MqttAcl {
            resource_type,
            resource_name,
            ip: "".to_string(),
            topic: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Deny,
        };

        let request = DeleteAclRequest {
            cluster_name: "test".to_string(),
            acl,
        };
        delete_acl(client_pool.clone(), addr.clone(), request).await;
    }

    async fn client_publish_test(client_id: &str, addr: &str, topic: &str) {
        let mqtt_version = 3;
        let cli = connect_server34(mqtt_version, client_id, addr);

        let msg = Message::new(topic.clone(), format!("mqtt message"), 0);
        match cli.publish(msg) {
            Ok(_) => {
                assert!("should not publish success");
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        distinct_conn(cli);
    }

    async fn client_subscribe_test(client_id: &str, addr: &str, topic: &str) {

    }
}
