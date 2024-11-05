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
    use grpc_clients::{
        mqtt::admin::call::{
            mqtt_broker_create_user, mqtt_broker_delete_user, mqtt_broker_list_user,
        },
        pool::ClientPool,
    };
    use metadata_struct::mqtt::user::MqttUser;
    use mqtt_broker::handler::connection::REQUEST_RESPONSE_PREFIX_NAME;
    use paho_mqtt::{Client, PropertyCode, ReasonCode};
    use protocol::broker_mqtt::broker_mqtt_admin::{
        CreateUserRequest, DeleteUserRequest, ListUserRequest,
    };

    use crate::common::{
        broker_addr, broker_grpc_addr, broker_ssl_addr, broker_ws_addr, broker_wss_addr,
        build_create_pros, build_v5_conn_pros_by_user_information, build_v5_pros, distinct_conn,
    };

    #[tokio::test]
    async fn client5_permission_test() {
        let client_id = unique_id();
        let addr = broker_addr();

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let username = "puser5".to_string();
        let password = "permission".to_string();

        //unregistered users are not allowed to create connections.
        v5_permission_wrong_test(
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );

        //registered users are allowed to create connections.
        create_user(
            client_pool.clone(),
            grpc_addr.clone(),
            username.clone(),
            password.clone(),
        )
        .await;
        v5_permission_success_test(
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );
        v5_response_test(
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );
        v5_assigned_client_id_test(&addr, username.clone(), password.clone(), false, false);
        v5_request_response_test(
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );

        //unregistered users are not allowed to create connections.
        delete_user(client_pool.clone(), grpc_addr.clone(), username.clone()).await;
        v5_permission_wrong_test(
            &client_id,
            &addr,
            username.clone(),
            password.clone(),
            false,
            false,
        );
    }

    fn v5_permission_wrong_test(
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

        let props = build_v5_pros();
        let conn_opts = build_v5_conn_pros_by_user_information(
            props,
            username.clone(),
            password.clone(),
            ws,
            ssl,
        );
        let err = cli.connect(conn_opts).unwrap_err();
        println!("Unable to connect:\n\t{:?}", err);
    }

    fn v5_permission_success_test(
        client_id: &str,
        addr: &str,
        username: String,
        password: String,
        ws: bool,
        ssl: bool,
    ) {
        let mqtt_version = 5;
        let props = build_v5_pros();

        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros_by_user_information(
            props.clone(),
            username.clone(),
            password.clone(),
            ws,
            ssl,
        );
        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                if ws {
                    if ssl {
                        assert_eq!(format!("wss://{}", resp.server_uri), broker_wss_addr());
                    } else {
                        assert_eq!(format!("ws://{}", resp.server_uri), broker_ws_addr());
                    }
                } else if ssl {
                    assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
                } else {
                    assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                }
                assert_eq!(mqtt_version, resp.mqtt_version);
                // assert!(!resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }

    fn v5_assigned_client_id_test(
        addr: &str,
        username: String,
        password: String,
        ws: bool,
        ssl: bool,
    ) {
        let mqtt_version = 5;
        let client_id = "".to_string();
        let props = build_v5_pros();

        let create_opts = build_create_pros(&client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros_by_user_information(
            props.clone(),
            username.clone(),
            password.clone(),
            ws,
            ssl,
        );
        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                if ws {
                    if ssl {
                        assert_eq!(format!("wss://{}", resp.server_uri), broker_wss_addr());
                    } else {
                        assert_eq!(format!("ws://{}", resp.server_uri), broker_ws_addr());
                    }
                } else if ssl {
                    assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
                } else {
                    assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                }
                assert_eq!(mqtt_version, resp.mqtt_version);
                // assert!(resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);

                let resp_pros = response.properties();
                let assign_client_id = resp_pros
                    .get(PropertyCode::AssignedClientIdentifer)
                    .unwrap()
                    .get_string()
                    .unwrap();
                assert!(!assign_client_id.is_empty());
                assert_eq!(assign_client_id.len(), unique_id().len());
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }

    fn v5_request_response_test(
        client_id: &str,
        addr: &str,
        username: String,
        password: String,
        ws: bool,
        ssl: bool,
    ) {
        let mqtt_version = 5;

        let pros = build_v5_pros();

        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros_by_user_information(
            pros.clone(),
            username.clone(),
            password.clone(),
            ws,
            ssl,
        );

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                // response
                if ws {
                    if ssl {
                        assert_eq!(format!("wss://{}", resp.server_uri), broker_wss_addr());
                    } else {
                        assert_eq!(format!("ws://{}", resp.server_uri), broker_ws_addr());
                    }
                } else if ssl {
                    assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
                } else {
                    assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                }
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert_eq!(response.reason_code(), ReasonCode::Success);

                // properties
                let resp_pros = response.properties();
                assert!(resp_pros
                    .get_string(PropertyCode::ResponseInformation)
                    .is_none());
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }

    fn v5_response_test(
        client_id: &str,
        addr: &str,
        username: String,
        password: String,
        ws: bool,
        ssl: bool,
    ) {
        let mqtt_version = 5;

        let mut pros = build_v5_pros();
        pros.push_val(PropertyCode::RequestResponseInformation, 1)
            .unwrap();

        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros_by_user_information(
            pros.clone(),
            username.clone(),
            password.clone(),
            ws,
            ssl,
        );

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                // response
                if ws {
                    if ssl {
                        assert_eq!(format!("wss://{}", resp.server_uri), broker_wss_addr());
                    } else {
                        assert_eq!(format!("ws://{}", resp.server_uri), broker_ws_addr());
                    }
                } else if ssl {
                    assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
                } else {
                    assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                }
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert_eq!(response.reason_code(), ReasonCode::Success);

                // properties
                let resp_pros = response.properties();
                assert_eq!(
                    resp_pros
                        .get(PropertyCode::SessionExpiryInterval)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    3
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::ReceiveMaximum)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    65535
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::MaximumQos)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    2
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::RetainAvailable)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::MaximumPacketSize)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    10485760
                );

                assert!(resp_pros
                    .get(PropertyCode::AssignedClientIdentifer)
                    .is_none());

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::TopicAliasMaximum)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    65535
                );

                assert!(resp_pros.get(PropertyCode::ReasonString).is_none());

                assert!(resp_pros.get(PropertyCode::UserProperty).is_none());

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::WildcardSubscriptionAvailable)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::SubscriptionIdentifiersAvailable)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::SharedSubscriptionAvailable)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::ServerKeepAlive)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1200
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::ResponseInformation)
                        .unwrap()
                        .get_string()
                        .unwrap(),
                    REQUEST_RESPONSE_PREFIX_NAME.to_string()
                );

                assert!(resp_pros.get(PropertyCode::ServerReference).is_none());

                assert!(resp_pros.get(PropertyCode::AuthenticationMethod).is_none());
                assert!(resp_pros.get(PropertyCode::AuthenticationData).is_none());
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
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
