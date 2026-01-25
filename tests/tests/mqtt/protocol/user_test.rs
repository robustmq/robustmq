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
    use crate::mqtt::protocol::common::{
        broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
        ssl_by_type, ws_by_type,
    };
    use crate::mqtt::protocol::ClientTestProperties;
    use admin_server::client::AdminHttpClient;
    use admin_server::mqtt::user::{CreateUserReq, DeleteUserReq, UserListReq, UserListRow};
    use admin_server::tool::PageReplyData;
    use common_base::tools::unique_id;

    #[tokio::test]
    async fn login_auth_test() {
        let admin_client = create_test_env().await;
        let network = "tcp";
        let client_id = build_client_id(format!("login_auth_test_{network}").as_str());
        let username = unique_id();
        let password = unique_id();

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            user_name: username.clone(),
            password: password.clone(),
            conn_is_err: true,
            ..Default::default()
        };
        connect_server(&client_properties);

        create_user(&admin_client, username.clone(), password.clone()).await;
        assert_eq!(get_user(&admin_client, username.clone()).await, 1);

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            user_name: username.clone(),
            password: password.clone(),
            conn_is_err: false,
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        distinct_conn(cli);

        delete_user(&admin_client, username.clone()).await;
        assert_eq!(get_user(&admin_client, username.clone()).await, 0);

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            user_name: username.clone(),
            password: password.clone(),
            conn_is_err: true,
            ..Default::default()
        };
        connect_server(&client_properties);
    }

    async fn create_user(admin_client: &AdminHttpClient, username: String, password: String) {
        let user = CreateUserReq {
            username: username.clone(),
            password,
            is_superuser: false,
        };
        admin_client.create_user(&user).await.unwrap();
    }

    async fn get_user(admin_client: &AdminHttpClient, username: String) -> usize {
        let user = UserListReq {
            user_name: Some(username),
            ..Default::default()
        };
        let res: PageReplyData<Vec<UserListRow>> = admin_client.get_user_list(&user).await.unwrap();
        res.total_count
    }

    async fn delete_user(admin_client: &AdminHttpClient, username: String) {
        let user = DeleteUserReq { username };
        admin_client.delete_user(&user).await.unwrap();
    }
}
