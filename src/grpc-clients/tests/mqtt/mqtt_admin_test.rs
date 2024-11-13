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

    use grpc_clients::mqtt::admin::call::{
        cluster_status, mqtt_broker_create_user, mqtt_broker_delete_user, mqtt_broker_list_user,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::user::MqttUser;
    use protocol::broker_mqtt::broker_mqtt_admin::{
        ClusterStatusRequest, CreateUserRequest, DeleteUserRequest, ListUserRequest,
    };

    use crate::common::get_mqtt_broker_addr;

    #[tokio::test]
    async fn cluster_status_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        let request = ClusterStatusRequest {};
        match cluster_status(client_pool.clone(), addrs.clone(), request).await {
            Ok(data) => {
                println!("{:?}", data);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn user_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];
        let user_name: String = "user1".to_string();
        let password: String = "123456".to_string();

        let user = CreateUserRequest {
            username: user_name.clone(),
            password: password.clone(),
            is_superuser: false,
        };

        match mqtt_broker_create_user(client_pool.clone(), addrs.clone(), user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        match mqtt_broker_list_user(client_pool.clone(), addrs.clone(), ListUserRequest {}).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(raw.as_slice()).unwrap();
                    if user.username == mqtt_user.username {
                        flag = true;
                    }
                }
                assert!(flag, "user1 has been created");
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        };

        match mqtt_broker_delete_user(
            client_pool.clone(),
            addrs.clone(),
            DeleteUserRequest {
                username: user.username.clone(),
            },
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        match mqtt_broker_list_user(client_pool.clone(), addrs.clone(), ListUserRequest {}).await {
            Ok(data) => {
                let mut flag = true;
                for raw in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(raw.as_slice()).unwrap();
                    if user.username == mqtt_user.username {
                        flag = false;
                    }
                }
                assert!(flag, "user1 should be deleted");
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        };
    }
}
