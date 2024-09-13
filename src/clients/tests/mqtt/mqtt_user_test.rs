// Copyright 2023 RobustMQ Team
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
    use crate::common::get_placement_addr;
    use clients::{
        placement::mqtt::call::{placement_list_user, placement_create_user,
            placement_delete_user},
        poll::ClientPool,
    };
    use metadata_struct::mqtt::user::MQTTUser;
    use protocol::placement_center::generate::mqtt::{
        ListUserRequest, CreateUserRequest,
        DeleteUserRequest,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn mqtt_user_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let user_name: String = "chungka".to_string();
        let password: String = "123456".to_string();
        let cluster_name: String = "test_cluster".to_string();
        
        let mqtt_user: MQTTUser = MQTTUser{
            username: user_name.clone(),
            password: password.clone(),
            is_superuser: false,
        };
        
        let request: CreateUserRequest = CreateUserRequest {
            cluster_name: cluster_name.clone(),
            user_name: mqtt_user.username.clone(),
            content: mqtt_user.encode(),
        };
        match placement_create_user(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request: ListUserRequest = ListUserRequest {
            cluster_name: cluster_name.clone(),
            user_name: mqtt_user.username.clone(),
        };

        match placement_list_user(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.users {
                    let user = serde_json::from_slice::<MQTTUser>(raw.as_slice()).unwrap();
                    if mqtt_user == user {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request: DeleteUserRequest = DeleteUserRequest {
            cluster_name: cluster_name.clone(),
            user_name: mqtt_user.username.clone(),
        };

        match placement_delete_user(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request: ListUserRequest = ListUserRequest {
            cluster_name: cluster_name.clone(),
            user_name: mqtt_user.username.clone(),
        };

        match placement_list_user(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.users {
                    let user = serde_json::from_slice::<MQTTUser>(raw.as_slice()).unwrap();
                    if mqtt_user == user {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }
}