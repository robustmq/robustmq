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

    use common_base::tools::now_second;
    use common_base::uuid::unique_id;
    use grpc_clients::meta::mqtt::call::{
        placement_create_user, placement_delete_user, placement_list_user,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::auth::user::SecurityUser;
    use protocol::meta::meta_service_mqtt::{
        CreateUserRequest, DeleteUserRequest, ListUserRequest,
    };

    use crate::common::{get_placement_addr, wait_until};

    #[tokio::test]

    async fn mqtt_user_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let user_name: String = unique_id();
        let password: String = "123456".to_string();

        let mqtt_user: SecurityUser = SecurityUser {
            tenant: "default".to_string(),
            username: user_name.clone(),
            password: password.clone(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };

        let request: CreateUserRequest = CreateUserRequest {
            tenant: "default".to_string(),
            user_name: mqtt_user.username.clone(),
            content: mqtt_user.encode().unwrap(),
        };
        placement_create_user(&client_pool, &addrs, request)
            .await
            .unwrap();

        let present = wait_until(|| async {
            let request = ListUserRequest {
                tenant: "default".to_string(),
                user_name: mqtt_user.username.clone(),
            };
            match placement_list_user(&client_pool, &addrs, request).await {
                Ok(data) => data
                    .users
                    .iter()
                    .filter_map(|raw| SecurityUser::decode(raw).ok())
                    .any(|user| mqtt_user == user),
                Err(_) => false,
            }
        })
        .await;
        assert!(present, "created user {} not visible", mqtt_user.username);

        let request: DeleteUserRequest = DeleteUserRequest {
            tenant: "default".to_string(),
            user_name: mqtt_user.username.clone(),
        };

        placement_delete_user(&client_pool, &addrs, request)
            .await
            .unwrap();

        let absent = wait_until(|| async {
            let request = ListUserRequest {
                tenant: "default".to_string(),
                user_name: mqtt_user.username.clone(),
            };
            match placement_list_user(&client_pool, &addrs, request).await {
                Ok(data) => !data
                    .users
                    .iter()
                    .filter_map(|raw| SecurityUser::decode(raw).ok())
                    .any(|user| mqtt_user == user),
                Err(_) => false,
            }
        })
        .await;
        assert!(absent, "deleted user {} still visible", mqtt_user.username);
    }
}
