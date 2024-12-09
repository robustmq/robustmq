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

use std::process;
use std::time::Duration;

use paho_mqtt::{
    Client, ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder,
    DisconnectOptionsBuilder, Properties, PropertyCode, ReasonCode, SslOptionsBuilder,
};

#[allow(dead_code)]
pub fn broker_addr() -> String {
    "tcp://127.0.0.1:1883".to_string()
}

#[allow(dead_code)]
pub fn broker_ssl_addr() -> String {
    "mqtts://127.0.0.1:8883".to_string()
}

#[allow(dead_code)]
pub fn broker_ws_addr() -> String {
    "ws://127.0.0.1:8093".to_string()
}

#[allow(dead_code)]
pub fn broker_wss_addr() -> String {
    "wss://127.0.0.1:8094".to_string()
}
#[allow(dead_code)]
pub fn broker_grpc_addr() -> String {
    "127.0.0.1:9981".to_string()
}

#[allow(dead_code)]
pub fn username() -> String {
    "admin".to_string()
}

#[allow(dead_code)]
pub fn password() -> String {
    "pwd123".to_string()
}

#[allow(dead_code)]
pub fn err_password() -> String {
    "pwd1235".to_string()
}

#[allow(dead_code)]
pub fn build_v5_pros() -> Properties {
    let mut props = Properties::new();
    props
        .push_u32(PropertyCode::SessionExpiryInterval, 3)
        .unwrap();
    props.push_u16(PropertyCode::ReceiveMaximum, 128).unwrap();
    props
        .push_u32(PropertyCode::MaximumPacketSize, 2048)
        .unwrap();
    props
        .push_u16(PropertyCode::TopicAliasMaximum, 128)
        .unwrap();
    props
        .push_val(PropertyCode::RequestResponseInformation, 0)
        .unwrap();
    props
        .push_val(PropertyCode::RequestProblemInformation, 1)
        .unwrap();
    props
        .push_string_pair(PropertyCode::UserProperty, "lobo1", "1")
        .unwrap();
    props
        .push_string_pair(PropertyCode::UserProperty, "lobo2", "2")
        .unwrap();
    props
}

#[allow(dead_code)]
pub fn build_v5_conn_pros(props: Properties, err_pwd: bool, ws: bool, ssl: bool) -> ConnectOptions {
    let pwd = if err_pwd { err_password() } else { password() };
    let mut conn_opts = if ws {
        ConnectOptionsBuilder::new_ws_v5()
    } else {
        ConnectOptionsBuilder::new_v5()
    };
    if ssl {
        let ssl_opts = SslOptionsBuilder::new()
            .trust_store(format!(
                "{}/../config/example/certs/ca.pem",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap()
            .verify(false)
            .disable_default_trust_store(false)
            .finalize();
        conn_opts.ssl_options(ssl_opts);
    }
    conn_opts
        .keep_alive_interval(Duration::from_secs(600))
        .clean_start(true)
        .connect_timeout(Duration::from_secs(60))
        .properties(props.clone())
        .user_name(username())
        .password(pwd)
        .finalize()
}

#[allow(dead_code)]
pub fn build_v5_conn_pros_by_user_information(
    props: Properties,
    username: String,
    password: String,
    ws: bool,
    ssl: bool,
) -> ConnectOptions {
    let mut conn_opts = if ws {
        ConnectOptionsBuilder::new_ws_v5()
    } else {
        ConnectOptionsBuilder::new_v5()
    };
    if ssl {
        let ssl_opts = SslOptionsBuilder::new()
            .trust_store(format!(
                "{}/../../config/example/certs/ca.pem",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap()
            .verify(false)
            .disable_default_trust_store(false)
            .finalize();
        conn_opts.ssl_options(ssl_opts);
    }
    conn_opts
        .keep_alive_interval(Duration::from_secs(600))
        .clean_start(true)
        .connect_timeout(Duration::from_secs(60))
        .properties(props.clone())
        .user_name(username)
        .password(password)
        .finalize()
}

#[allow(dead_code)]
pub fn build_v3_conn_pros(mqtt_version: u32, err_pwd: bool, ws: bool, ssl: bool) -> ConnectOptions {
    let pwd = if err_pwd { err_password() } else { password() };
    let mut conn_opts = if ws {
        ConnectOptionsBuilder::new_ws()
    } else {
        ConnectOptionsBuilder::with_mqtt_version(mqtt_version)
    };
    if ssl {
        let ssl_opts = SslOptionsBuilder::new()
            .trust_store(format!(
                "{}/../config/example/certs/ca.pem",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap()
            .verify(false)
            .disable_default_trust_store(false)
            .finalize();
        conn_opts.ssl_options(ssl_opts);
    }
    conn_opts
        .keep_alive_interval(Duration::from_secs(600))
        .clean_session(true)
        .connect_timeout(Duration::from_secs(50))
        .user_name(username())
        .password(pwd)
        .finalize()
}

#[allow(dead_code)]
pub fn build_v3_conn_pros_by_user_information(
    mqtt_version: u32,
    username: String,
    password: String,
    ws: bool,
    ssl: bool,
) -> ConnectOptions {
    let mut conn_opts = if ws {
        ConnectOptionsBuilder::new_ws()
    } else {
        ConnectOptionsBuilder::with_mqtt_version(mqtt_version)
    };
    if ssl {
        let ssl_opts = SslOptionsBuilder::new()
            .trust_store(format!(
                "{}/../../config/example/certs/ca.pem",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap()
            .verify(false)
            .disable_default_trust_store(false)
            .finalize();
        conn_opts.ssl_options(ssl_opts);
    }
    conn_opts
        .keep_alive_interval(Duration::from_secs(600))
        .clean_session(true)
        .connect_timeout(Duration::from_secs(50))
        .user_name(username)
        .password(password)
        .finalize()
}

#[allow(dead_code)]
pub fn build_create_pros(client_id: &str, addr: &str) -> CreateOptions {
    if client_id.is_empty() {
        CreateOptionsBuilder::new().server_uri(addr).finalize()
    } else {
        CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id)
            .finalize()
    }
}

#[allow(dead_code)]
pub fn distinct_conn(cli: Client) {
    let disconnect_opts = DisconnectOptionsBuilder::new()
        .reason_code(ReasonCode::DisconnectWithWillMessage)
        .finalize();
    cli.disconnect(disconnect_opts).unwrap();
}

#[allow(dead_code)]
pub fn connect_server34(
    mqtt_version: u32,
    client_id: &str,
    addr: &str,
    ws: bool,
    ssl: bool,
) -> Client {
    let create_opts = build_create_pros(client_id, addr);
    let cli = Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

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
            } else if ssl {
                assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
            } else {
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
            }
            assert_eq!(mqtt_version, resp.mqtt_version);
            assert_eq!(response.reason_code(), ReasonCode::Success);
        }
        Err(e) => {
            println!("Unable to connect:\n\t{:?}", e);
            process::exit(1);
        }
    }
    cli
}

#[allow(dead_code)]
pub fn connect_server5(client_id: &str, addr: &str, ws: bool, ssl: bool) -> Client {
    let mqtt_version = 5;
    let props = build_v5_pros();

    let create_opts = build_create_pros(client_id, addr);
    let cli = Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let conn_opts = build_v5_conn_pros(props.clone(), false, ws, ssl);
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
            assert_eq!(response.reason_code(), ReasonCode::Success);
        }
        Err(e) => {
            println!("Unable to connect:\n\t{:?}", e);
            process::exit(1);
        }
    }
    cli
}

#[allow(dead_code)]
pub fn connect_server5_by_user_information(
    client_id: &str,
    addr: &str,
    username: String,
    password: String,
    ws: bool,
    ssl: bool,
) -> Client {
    let mqtt_version = 5;
    let props = build_v5_pros();

    let create_opts = build_create_pros(client_id, addr);
    let cli = Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let conn_opts =
        build_v5_conn_pros_by_user_information(props.clone(), username, password, ws, ssl);
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
            assert_eq!(response.reason_code(), ReasonCode::Success);
        }
        Err(e) => {
            println!("Unable to connect:\n\t{:?}", e);
            process::exit(1);
        }
    }
    cli
}

#[allow(dead_code)]
pub fn connect_server5_packet_size(
    client_id: &str,
    addr: &str,
    packet_size: i32,
    ws: bool,
    ssl: bool,
) -> Client {
    let mqtt_version = 5;
    let mut props = build_v5_pros();
    props
        .push_int(PropertyCode::MaximumPacketSize, packet_size)
        .unwrap();

    let create_opts = build_create_pros(client_id, addr);
    let cli = Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let conn_opts = build_v5_conn_pros(props.clone(), false, ws, ssl);
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
            assert!(resp.session_present);
            assert_eq!(response.reason_code(), ReasonCode::Success);
        }
        Err(e) => {
            println!("Unable to connect:\n\t{:?}", e);
            process::exit(1);
        }
    }
    cli
}

#[allow(dead_code)]
pub fn connect_server5_response_information(client_id: &str, addr: &str) -> (Client, String) {
    let mqtt_version = 5;
    let mut props = build_v5_pros();
    props
        .push_val(PropertyCode::RequestResponseInformation, 1)
        .unwrap();

    let create_opts = build_create_pros(client_id, addr);
    let cli = Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let conn_opts = build_v5_conn_pros(props.clone(), false, false, false);
    let response_information = match cli.connect(conn_opts) {
        Ok(response) => {
            let resp = response.connect_response().unwrap();

            assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
            assert_eq!(mqtt_version, resp.mqtt_version);
            assert!(resp.session_present);
            assert_eq!(response.reason_code(), ReasonCode::Success);

            let resp_pros = response.properties();
            resp_pros
                .get_string(PropertyCode::ResponseInformation)
                .unwrap()
        }
        Err(e) => {
            println!("Unable to connect:\n\t{:?}", e);
            process::exit(1);
        }
    };
    (cli, response_information)
}
