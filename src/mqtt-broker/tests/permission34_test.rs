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
// extern crate mqtt_broker;

#[cfg(test)]
mod tests {
    use std::{
         net::{Ipv4Addr, SocketAddrV4}, process, sync::Arc, thread::{self, Thread}, time::Duration
    };

    use axum_extra::headers::Date;
    use common_base::{config::broker_mqtt::broker_mqtt_conf, tools::unique_id};
    use grpc_clients::{
        mqtt::admin::call::{
            mqtt_broker_create_user, mqtt_broker_delete_user, mqtt_broker_list_user,
        },
        pool::ClientPool,
    };
    use metadata_struct::mqtt::user::MqttUser;
    use mqtt_broker::{handler::cache::CacheManager, security::AuthDriver};
    use paho_mqtt::{Client, ReasonCode};
    use protocol::{broker_mqtt::broker_mqtt_admin::{
        CreateUserRequest, DeleteUserRequest, ListUserRequest,
    }, mqtt::common::Login};
    use std::net::SocketAddr;

    use crate::common::{
        broker_addr, broker_grpc_addr, broker_ssl_addr, broker_ws_addr, broker_wss_addr,
        build_create_pros, build_v3_conn_pros_by_user_information, distinct_conn,
    };

    #[tokio::test]
    async fn client3_permission_test(){
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let conf = broker_mqtt_conf();
        let cache_manager: Arc<CacheManager> = Arc::new(CacheManager::new(
            client_pool.clone(),
            conf.cluster_name.clone(),
        ));

        let auth_driver = Arc::new(AuthDriver::new(cache_manager.clone(), client_pool.clone()));

        let username = "puser3".to_string();
        let password = "permission".to_string();

        //unregistered users are not allowed to create connections.
        v3_wrong_password_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);

        create_user(client_pool.clone(), grpc_addr.clone(), username.clone(), password.clone()).await;

        //registered users are allowed to create connections.
        v3_session_present_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);

        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;

        //unregistered users are not allowed to create connections.

        v3_wrong_password_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);
    }


    #[tokio::test]
    async fn client4_permission_test(){
        let mqtt_version = 4;
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = "puser4".to_string();
        let password = "permission".to_string();

        //unregistered users are not allowed to create connections.
        v3_wrong_password_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);

        create_user(client_pool.clone(), grpc_addr.clone(), username.clone(), password.clone()).await;

        //registered users are allowed to create connections.
        v3_session_present_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);

        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;

        //unregistered users are not allowed to create connections.
        v3_wrong_password_test(mqtt_version, &client_id, &addr, username.clone(), password.clone(), false, false);
    }

    async fn permission_success(auth_driver: &AuthDriver, username: String, password: String){
        let socket = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));

        let login = Some(Login {
            username: username.clone(),
            password: password.clone(),
        });

        let registered_user_permission_result = auth_driver.check_login_auth(&login,&None,&socket).await;

        match registered_user_permission_result {
            Ok(value) =>{
                assert!(value,"registed user should pass permission");
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    async fn permission_failture(auth_driver: &AuthDriver, username: String, password: String){
        let socket = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));

        let login = Some(Login {
            username: username.clone(),
            password: password.clone(),
        });

        let unregistered_user_permission_result = auth_driver.check_login_auth(&login,&None,&socket).await;

        match unregistered_user_permission_result {
            Ok(value) =>{
                assert!(!value,"unregisted user should not pass permission");
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }


    #[tokio::test]
    async fn exist_user_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = "user5".to_string();
        let password = "555555".to_string();

        //判断用户是否存在
        let is_user_exist =
            estimate_user_exist(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
        println!("user2此时是否存在？ {}", is_user_exist);

        v3_session_present_test(
            mqtt_version,
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );
    }

    #[tokio::test]
    async fn unexist_user_perssion_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();

        let username = "user5".to_string();
        let password = "555555".to_string();

        v3_wrong_password_test(
            mqtt_version,
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );
    }

    #[tokio::test]
    async fn create_user_perssion_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = "user5".to_string();
        let password = "555555".to_string();

        create_user(
            client_pool.clone(),
            grpc_addr.clone(),
            username.clone(),
            password.clone(),
        )
        .await;

        v3_session_present_test(
            mqtt_version,
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );
    }

    #[tokio::test]
    async fn dele_user_perssion_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = "user5".to_string();
        let password = "555555".to_string();

        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;

        v3_wrong_password_test(
            mqtt_version,
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );
    }

    #[tokio::test]
    async fn user_information_perssion_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = "user2".to_string();
        let password = "1234567".to_string();

        //判断用户是否存在
        let is_user_exist =
            estimate_user_exist(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
        println!("user2此时是否存在？ {}", is_user_exist);

        match is_user_exist {
            true => {
                v3_session_present_test(
                    mqtt_version,
                    &client_id,
                    &addr,
                    username.clone(),
                    password.clone(),
                    false,
                    false,
                );

                delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;

                let is_user_exist =
                    estimate_user_exist(client_pool.clone(), grpc_addr.clone(), username.clone())
                        .await;
                println!("user2删除后是否存在？ {}", is_user_exist);

                v3_wrong_password_test(
                    mqtt_version,
                    &client_id,
                    &addr,
                    username.clone(),
                    password.clone(),
                    false,
                    false,
                );
                return;
            }
            false => {
                v3_wrong_password_test(
                    mqtt_version,
                    &client_id,
                    &addr,
                    username.clone(),
                    password.clone(),
                    false,
                    false,
                );

                create_user(
                    client_pool.clone(),
                    grpc_addr.clone(),
                    username.clone(),
                    password.clone(),
                )
                .await;

                let is_user_exist =
                    estimate_user_exist(client_pool.clone(), grpc_addr.clone(), username.clone())
                        .await;
                println!("user2新增后是否存在？ {}", is_user_exist);

                v3_session_present_test(
                    mqtt_version,
                    &client_id,
                    &addr,
                    username.clone(),
                    password.clone(),
                    false,
                    false,
                );

                delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;

                let is_user_exist =
                    estimate_user_exist(client_pool.clone(), grpc_addr.clone(), username.clone())
                        .await;
                println!("user2删除后是否存在？ {}", is_user_exist);

                thread::sleep(Duration::from_secs(55));

                v3_wrong_password_test(
                    mqtt_version,
                    &client_id,
                    &addr,
                    username.clone(),
                    password.clone(),
                    false,
                    false,
                );
            }
        }
    }

    fn v3_wrong_password_test(
        mqtt_version: u32,
        client_id: &str,
        addr: &str,
        username: String,
        password: String,
        ws: bool,
        ssl: bool,
    ) {
        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts =
            build_v3_conn_pros_by_user_information(mqtt_version, username, password, ws, ssl);

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

        let conn_opts = build_v3_conn_pros_by_user_information(
            mqtt_version,
            username.clone(),
            password.clone(),
            ws,
            ssl,
        );
        println!("{:?}", conn_opts);
        let response = cli.connect(conn_opts).unwrap();
        let resp = response.connect_response().unwrap();
        println!("{:?}", resp);
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

        let conn_opts = build_v3_conn_pros_by_user_information(
            mqtt_version,
            username.clone(),
            password.clone(),
            ws,
            ssl,
        );

        let response = cli.connect(conn_opts).unwrap();
        let resp = response.connect_response().unwrap();
        println!("{:?}", resp);
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

    async fn estimate_user_exist(
        client_pool: Arc<ClientPool>,
        addrs: Vec<String>,
        username: String,
    ) -> bool {
        match mqtt_broker_list_user(client_pool.clone(), addrs.clone(), ListUserRequest {}).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(raw.as_slice()).unwrap();
                    if username == mqtt_user.username {
                        flag = true;
                    }
                }
                flag
            }
            Err(e) => {
                panic!("{:?}", e);
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
        match mqtt_broker_create_user(client_pool.clone(), addrs.clone(), user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    async fn delete_user(client_pool: Arc<ClientPool>, addrs: Vec<String>, username: String) {
        let user = DeleteUserRequest { username };
        match mqtt_broker_delete_user(client_pool.clone(), addrs.clone(), user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
