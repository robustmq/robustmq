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

    use common_base::tools::unique_id;
    use grpc_clients::mqtt::admin::call::{mqtt_broker_create_user, mqtt_broker_delete_user};
    use grpc_clients::pool::ClientPool;
    use protocol::broker_mqtt::broker_mqtt_admin::{CreateUserRequest, DeleteUserRequest};

    use crate::mqtt_protocol::common::{
        broker_addr_by_type, broker_grpc_addr, build_client_id, connect_server, distinct_conn,
        network_types, protocol_versions, qos_list, ssl_by_type, ws_by_type,
    };
    use crate::mqtt_protocol::ClientTestProperties;

    #[tokio::test]
    async fn login_auth_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        for protocol in protocol_versions() {
            for network in network_types() {
                for qos in qos_list() {
                    let client_id =
                        build_client_id(format!("publish_qos_test_{}_{}", network, qos).as_str());

                    let username = unique_id();
                    let password = "permission".to_string();

                    let client_properties = ClientTestProperties {
                        mqtt_version: protocol,
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

                    create_user(
                        client_pool.clone(),
                        grpc_addr.clone(),
                        username.clone(),
                        password.clone(),
                    )
                    .await;

                    let client_properties = ClientTestProperties {
                        mqtt_version: protocol,
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

                    delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;

                    let client_properties = ClientTestProperties {
                        mqtt_version: protocol,
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
        }
    }

    async fn create_user(
        client_pool: Arc<ClientPool>,
        addrs: Vec<String>,
        username: String,
        password: String,
    ) {
        let user = CreateUserRequest {
            username,
            password,
            is_superuser: false,
        };
        let res = mqtt_broker_create_user(&client_pool, &addrs, user.clone()).await;
        assert!(res.is_ok());
    }

    async fn delete_user(client_pool: Arc<ClientPool>, addrs: Vec<String>, username: String) {
        let user = DeleteUserRequest { username };
        let res = mqtt_broker_delete_user(&client_pool, &addrs, user.clone()).await;
        assert!(res.is_ok());
    }
}
