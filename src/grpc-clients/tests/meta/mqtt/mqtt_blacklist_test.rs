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
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::tools::now_second;
    use grpc_clients::meta::mqtt::call::{create_blacklist, delete_blacklist, list_blacklist};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
    use protocol::meta::meta_service_mqtt::{
        CreateBlacklistRequest, DeleteBlacklistRequest, ListBlacklistRequest,
    };
    use std::sync::Arc;

    use crate::common::get_placement_addr;

    #[tokio::test]

    async fn mqtt_blacklist_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];

        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: "loboxu".to_string(),
            end_time: now_second() + 100,
            desc: "loboxu test".to_string(),
        };

        let request = CreateBlacklistRequest {
            blacklist: blacklist.encode().unwrap(),
        };
        match create_blacklist(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let request = ListBlacklistRequest {
        };

        match list_blacklist(&client_pool, &addrs, request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.blacklists {
                    let tmp = MqttAclBlackList::decode(&raw).unwrap();
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
                panic!("{e:?}");
            }
        }

        let request = DeleteBlacklistRequest {
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name.clone(),
        };
        match delete_blacklist(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let request = ListBlacklistRequest {
        };

        match list_blacklist(&client_pool, &addrs, request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.blacklists {
                    let tmp = MqttAclBlackList::decode(&raw).unwrap();
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
                panic!("{e:?}");
            }
        }
    }
}
