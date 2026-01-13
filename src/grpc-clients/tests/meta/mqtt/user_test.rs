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

    use common_base::tools::{now_second, unique_id};
    use grpc_clients::meta::mqtt::call::{
        placement_create_user, placement_delete_user, placement_list_user,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::user::MqttUser;
    use protocol::meta::meta_service_mqtt::{
        CreateUserRequest, DeleteUserRequest, ListUserRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]

    async fn mqtt_user_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let user_name: String = unique_id();
        let password: String = "123456".to_string();

        let mqtt_user: MqttUser = MqttUser {
            username: user_name.clone(),
            password: password.clone(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };

        let request: CreateUserRequest = CreateUserRequest {
            user_name: mqtt_user.username.clone(),
            content: mqtt_user.encode().unwrap(),
        };
        placement_create_user(&client_pool, &addrs, request)
            .await
            .unwrap();

        let request: ListUserRequest = ListUserRequest {
            user_name: mqtt_user.username.clone(),
        };

        let data = placement_list_user(&client_pool, &addrs, request)
            .await
            .unwrap();

        let mut flag: bool = false;
        for raw in data.users {
            let user = MqttUser::decode(&raw).unwrap();
            if mqtt_user == user {
                flag = true;
            }
        }
        assert!(flag);

        let request: DeleteUserRequest = DeleteUserRequest {
            user_name: mqtt_user.username.clone(),
        };

        placement_delete_user(&client_pool, &addrs, request)
            .await
            .unwrap();

        let request: ListUserRequest = ListUserRequest {
            user_name: mqtt_user.username.clone(),
        };

        let data = placement_list_user(&client_pool, &addrs, request)
            .await
            .unwrap();

        let mut flag: bool = false;
        for raw in data.users {
            let user = MqttUser::decode(&raw).unwrap();
            if mqtt_user == user {
                flag = true;
            }
        }
        assert!(!flag);
    }
}
