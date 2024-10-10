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
    use std::process;

    use common_base::tools::unique_id;
    use paho_mqtt::{Client, ReasonCode};

    use crate::common::{
        broker_addr, broker_ssl_addr, broker_ws_addr, broker_wss_addr, build_create_pros,
        build_v3_conn_pros, distinct_conn,
    };

    #[tokio::test]
    async fn client34_connect_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr, false, false);
        v3_session_present_test(mqtt_version, &client_id, &addr, false, false);

        let mqtt_version = 4;
        let client_id = unique_id();
        let addr = broker_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr, false, false);
        v3_session_present_test(mqtt_version, &client_id, &addr, false, false);
    }

    #[tokio::test]
    async fn client34_connect_ssl_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_ssl_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr, false, true);
        v3_session_present_test(mqtt_version, &client_id, &addr, false, true);

        let mqtt_version = 4;
        let client_id = unique_id();
        let addr = broker_ssl_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr, false, true);
        v3_session_present_test(mqtt_version, &client_id, &addr, false, true);
    }

    #[tokio::test]
    async fn client4_connect_ws_test() {
        let mqtt_version = 4;
        let client_id = unique_id();
        let addr = broker_ws_addr();

        v3_wrong_password_test(mqtt_version, &client_id, &addr, true, false);
        // v3_session_present_test(mqtt_version, &client_id, &addr, true, false);
    }

    #[tokio::test]
    #[ignore]
    async fn client4_connect_wss_test() {
        let mqtt_version = 4;
        let client_id = unique_id();
        let addr = broker_wss_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr, true, true);
        v3_session_present_test(mqtt_version, &client_id, &addr, true, true);
    }

    fn v3_wrong_password_test(
        mqtt_version: u32,
        client_id: &String,
        addr: &String,
        ws: bool,
        ssl: bool,
    ) {
        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v3_conn_pros(mqtt_version, true, ws, ssl);

        match cli.connect(conn_opts) {
            Ok(_) => {
                assert!(false)
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                assert!(true)
            }
        }
    }

    fn v3_session_present_test(
        mqtt_version: u32,
        client_id: &String,
        addr: &String,
        ws: bool,
        ssl: bool,
    ) {
        let create_opts = build_create_pros(client_id, addr);
        let cli = match Client::new(create_opts) {
            Ok(data) => data,
            Err(e) => {
                println!("{}", e);
                assert!(false);
                return;
            }
        };

        let conn_opts = build_v3_conn_pros(mqtt_version, false, ws, ssl);
        println!("{:?}", conn_opts);
        match cli.connect(conn_opts) {
            Ok(response) => {
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
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                assert!(false);
                return;
            }
        }
        distinct_conn(cli);

        let create_opts = build_create_pros(client_id, addr);

        let cli = match Client::new(create_opts) {
            Ok(data) => data,
            Err(e) => {
                println!("Error creating the client: {:?}", e);
                assert!(false);
                return;
            }
        };

        let conn_opts = build_v3_conn_pros(mqtt_version, false, ws, ssl);

        match cli.connect(conn_opts) {
            Ok(response) => {
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
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                assert!(false);
                return;
            }
        }

        distinct_conn(cli);
    }
}
