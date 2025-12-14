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
    use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
    use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
    use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
    use grpc_clients::meta::mqtt::call::{create_acl, delete_acl, list_acl};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::acl::mqtt_acl::MqttAcl;
    use protocol::meta::meta_service_mqtt::{CreateAclRequest, DeleteAclRequest, ListAclRequest};
    use std::sync::Arc;

    use crate::common::get_placement_addr;

    #[tokio::test]
    async fn mqtt_acl_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: "loboxu".to_string(),
            topic: "tp-1".to_string(),
            ip: "*".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Deny,
        };

        let request = CreateAclRequest {
            acl: acl.encode().unwrap(),
        };
        create_acl(&client_pool, &addrs, request).await.unwrap();

        let request = ListAclRequest {};

        match list_acl(&client_pool, &addrs, request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.acls {
                    let tmp = MqttAcl::decode(&raw).unwrap();
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
                panic!("{e:?}");
            }
        }

        let request = DeleteAclRequest {
            acl: acl.encode().unwrap(),
        };
        match delete_acl(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let request = ListAclRequest {};

        match list_acl(&client_pool, &addrs, request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.acls {
                    let tmp = MqttAcl::decode(&raw).unwrap();
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
                panic!("{e:?}");
            }
        }
    }
}
