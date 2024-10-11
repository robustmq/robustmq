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

    use clients::placement::mqtt::call::{create_acl, delete_acl, list_acl, list_blacklist};
    use clients::poll::ClientPool;
    use common_base::tools::unique_id;
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };
    use protocol::placement_center::generate::mqtt::{
        CreateAclRequest, DeleteAclRequest, ListAclRequest, ListBlacklistRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]
    async fn mqtt_acl_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let cluster_name: String = format!("test_cluster_{}", unique_id());

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: "loboxu".to_string(),
            topic: "tp-1".to_string(),
            ip: "*".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Deny,
        };

        let request = CreateAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };
        create_acl(client_poll.clone(), addrs.clone(), request)
            .await
            .unwrap();

        let request = ListAclRequest {
            cluster_name: cluster_name.clone(),
        };

        match list_acl(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag = false;
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
                panic!("{:?}", e);
            }
        }

        let request = DeleteAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };
        match delete_acl(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = ListBlacklistRequest {
            cluster_name: cluster_name.clone(),
        };

        match list_blacklist(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.blacklists {
                    let tmp = serde_json::from_slice::<MqttAcl>(raw.as_slice())
                        .unwrap_or_else(|e| panic!("{e} {:02x?}", raw.as_slice()));
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
                panic!("{:?}", e);
            }
        }
    }
}
