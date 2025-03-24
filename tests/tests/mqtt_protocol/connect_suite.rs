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

use crate::mqtt_protocol::common::{
    broker_addr, broker_ssl_addr, broker_ws_addr, broker_wss_addr, build_conn_pros,
    build_create_pros, distinct_conn,
};
use paho_mqtt::{Client, ReasonCode};

#[derive(Debug, Clone)]
pub struct ClientTestProperties {
    pub(crate) mqtt_version: u32,
    pub(crate) client_id: String,
    pub(crate) addr: String,
    pub(crate) ws: bool,
    pub(crate) ssl: bool,
}

pub fn wrong_password_test(client_test_properties: ClientTestProperties) {
    let create_opts = build_create_pros(
        &client_test_properties.client_id,
        &client_test_properties.addr,
    );

    let cli_res = Client::new(create_opts);
    assert!(cli_res.is_ok());
    let cli = cli_res.unwrap();

    let conn_opts = build_conn_pros(client_test_properties.clone(), true);
    let result = cli.connect(conn_opts);
    println!(
        "client_test_properties:{:?},result:{:?}",
        client_test_properties, result
    );
    assert!(result.is_err());
}

pub fn create_session_connection(client_properties: &ClientTestProperties, _present: bool) {
    let create_opts = build_create_pros(&client_properties.client_id, &client_properties.addr);
    let cli = Client::new(create_opts).unwrap();

    let conn_opts = build_conn_pros(client_properties.clone(), false);
    let response = cli.connect(conn_opts).unwrap();

    let resp = response.connect_response().unwrap();
    if client_properties.ws {
        if client_properties.ssl {
            assert_eq!(format!("wss://{}", resp.server_uri), broker_wss_addr());
        } else {
            assert_eq!(format!("ws://{}", resp.server_uri), broker_ws_addr());
        }
    } else {
        if client_properties.ssl {
            assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
        } else {
            assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
        }
    }
    println!("client_properties:{:?},resp:{:?}", client_properties, resp);

    assert_eq!(client_properties.mqtt_version, resp.mqtt_version);
    assert_eq!(response.reason_code(), ReasonCode::Success);

    distinct_conn(cli);
}
