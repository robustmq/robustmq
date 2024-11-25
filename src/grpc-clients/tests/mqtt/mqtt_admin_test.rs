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
        cluster_status, mqtt_broker_create_user, mqtt_broker_delete_user,
        mqtt_broker_list_connection, mqtt_broker_list_user,
    };
    use grpc_clients::pool::ClientPool;
    use protocol::broker_mqtt::broker_mqtt_admin::{
        ClusterStatusRequest, CreateUserRequest, DeleteUserRequest, ListConnectionRequest,
        ListUserRequest,
    };

    use crate::common::get_mqtt_broker_addr;

    #[tokio::test]
    async fn cluster_status_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        let request = ClusterStatusRequest {};
        cluster_status(client_pool.clone(), addrs.clone(), request)
            .await
            .unwrap();
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

        mqtt_broker_create_user(client_pool.clone(), addrs.clone(), user.clone())
            .await
            .unwrap();

        mqtt_broker_list_user(client_pool.clone(), addrs.clone(), ListUserRequest {})
            .await
            .unwrap();

        mqtt_broker_delete_user(
            client_pool.clone(),
            addrs.clone(),
            DeleteUserRequest {
                username: user.username.clone(),
            },
        )
        .await
        .unwrap();

        mqtt_broker_list_user(client_pool.clone(), addrs.clone(), ListUserRequest {})
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_list_connection() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        mqtt_broker_list_connection(client_pool, addrs, ListConnectionRequest {})
            .await
            .unwrap();
    }
}
