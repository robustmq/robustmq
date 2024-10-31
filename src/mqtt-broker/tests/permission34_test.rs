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

mod common;

#[cfg(test)]
mod tests {
    use std::{process, sync::Arc};

    use common_base::tools::unique_id;
    use grpc_clients::{mqtt::admin::call::{mqtt_broker_create_user, mqtt_broker_delete_user, mqtt_broker_list_user}, pool::ClientPool};
    use metadata_struct::mqtt::user::MqttUser;
    use paho_mqtt::{Client, ReasonCode};
    use protocol::broker_mqtt::broker_mqtt_admin::{CreateUserRequest, DeleteUserRequest, ListUserRequest};

    use crate::common::{broker_addr, broker_grpc_addr, broker_ssl_addr, broker_ws_addr, broker_wss_addr, build_create_pros, build_v3_conn_pros_by_user_information, distinct_conn};

    #[tokio::test]
    async fn user_information_perssion34_test() {

        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = "user2".to_string();
        let password = "1234567".to_string();

        let is_user_exist = estimate_user_exist(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
        println!("user2此时是否存在？ {}",is_user_exist);

        v3_wrong_password_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);

        create_user(client_pool.clone(), grpc_addr.clone(), username.clone(), password.clone()).await;

        let is_user_exist = estimate_user_exist(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
        println!("user2新增后是否存在？ {}",is_user_exist);
        
        v3_session_present_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);

        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;

        let is_user_exist = estimate_user_exist(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
        println!("user2删除后是否存在？ {}",is_user_exist);

        v3_wrong_password_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);
        }

    fn v3_wrong_password_test(mqtt_version: u32, client_id: &str, addr: &str, username: String, password: String, ws: bool, ssl: bool) {
        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v3_conn_pros_by_user_information(mqtt_version, username, password, ws, ssl);

        let err = cli.connect(conn_opts).unwrap_err();
        println!("Unable to connect:\n\t{:?}", err);
    }

    fn v3_session_present_test(
        mqtt_version: u32,
        client_id: &str,
        addr: &str,
        username: String,
        password: String,
        ws: bool,
        ssl: bool,
    ) {
        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap();

        let conn_opts = build_v3_conn_pros_by_user_information(mqtt_version, username.clone(), password.clone(), ws, ssl);
        println!("{:?}", conn_opts);
        let response = cli.connect(conn_opts).unwrap();
        let resp = response.connect_response().unwrap();
        if ws {
            if ssl {
                assert_eq!(format!("wss://{}", resp.server_uri), broker_wss_addr());
            } else {
                assert_eq!(format!("ws://{}", resp.server_uri), broker_ws_addr());
            }
            assert_eq!(4, resp.mqtt_version);
        } else {
            if ssl {
                assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
            } else {
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
            }
            assert_eq!(mqtt_version, resp.mqtt_version);
        }
        assert!(resp.session_present);
        assert_eq!(response.reason_code(), ReasonCode::Success);
        distinct_conn(cli);

        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap();

        let conn_opts = build_v3_conn_pros_by_user_information(mqtt_version, username.clone(), password.clone(), ws, ssl);

        let response = cli.connect(conn_opts).unwrap();
        let resp = response.connect_response().unwrap();
        if ws {
            if ssl {
                assert_eq!(format!("wss://{}", resp.server_uri), broker_wss_addr());
            } else {
                assert_eq!(format!("ws://{}", resp.server_uri), broker_ws_addr());
            }
            assert_eq!(4, resp.mqtt_version);
        } else {
            if ssl {
                assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
            } else {
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
            }
            assert_eq!(mqtt_version, resp.mqtt_version);
        }
        assert!(!resp.session_present);
        assert_eq!(response.reason_code(), ReasonCode::Success);
        distinct_conn(cli);
    }

    async fn estimate_user_exist(client_pool: Arc<ClientPool>, addrs: Vec<String>, username: String) -> bool {
        match mqtt_broker_list_user(client_pool.clone(), addrs.clone(), ListUserRequest {}).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(raw.as_slice()).unwrap();
                    if username == mqtt_user.username {
                        flag = true;
                    }
                };
                flag
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    async fn create_user(client_pool: Arc<ClientPool>, addrs: Vec<String>, username: String, password: String) {
        let user = CreateUserRequest {
            username,
            password,
            is_superuser: false,
        };
        match mqtt_broker_create_user(client_pool.clone(), addrs.clone(), user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    async fn delete_user(client_pool: Arc<ClientPool>, addrs: Vec<String>, username: String) {
        let user = DeleteUserRequest {
            username,
        };
        match mqtt_broker_delete_user(client_pool.clone(), addrs.clone(), user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}