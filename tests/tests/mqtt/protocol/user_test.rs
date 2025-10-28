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
    use admin_server::client::AdminHttpClient;
    use admin_server::mqtt::user::{CreateUserReq, DeleteUserReq};
    use common_base::tools::unique_id;

    use crate::mqtt::protocol::common::{
        broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
        network_types, ssl_by_type, ws_by_type,
    };
    use crate::mqtt::protocol::ClientTestProperties;

    #[tokio::test]
    async fn login_auth_test() {
        for network in network_types() {
            let admin_client = create_test_env().await;
            let qos = 1;
            let client_id = build_client_id(format!("login_auth_test_{network}_{qos}").as_str());

            let username = unique_id();
            let password = "permission".to_string();

            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(&network),
                ws: ws_by_type(&network),
                ssl: ssl_by_type(&network),
                user_name: username.clone(),
                password: password.clone(),
                conn_is_err: true,
                ..Default::default()
            };
            connect_server(&client_properties);

            create_user(&admin_client, username.clone(), password.clone()).await;

            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(&network),
                ws: ws_by_type(&network),
                ssl: ssl_by_type(&network),
                user_name: username.clone(),
                password: password.clone(),
                conn_is_err: false,
                ..Default::default()
            };
            let cli = connect_server(&client_properties);
            distinct_conn(cli);

            delete_user(&admin_client, username.clone()).await;

            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id: client_id.to_string(),
                addr: broker_addr_by_type(&network),
                ws: ws_by_type(&network),
                ssl: ssl_by_type(&network),
                user_name: username.clone(),
                password: password.clone(),
                conn_is_err: true,
                ..Default::default()
            };
            connect_server(&client_properties);
        }
    }

    async fn create_user(admin_client: &AdminHttpClient, username: String, password: String) {
        let user = CreateUserReq {
            username,
            password,
            is_superuser: false,
        };
        let res = admin_client.create_user(&user).await;
        assert!(res.is_ok());
    }

    async fn delete_user(admin_client: &AdminHttpClient, username: String) {
        let user = DeleteUserReq { username };
        let res = admin_client.delete_user(&user).await;
        assert!(res.is_ok());
    }
}
