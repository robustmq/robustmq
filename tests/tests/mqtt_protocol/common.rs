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

use std::time::Duration;

use crate::mqtt_protocol::ClientTestProperties;
use common_base::tools::unique_id;
use paho_mqtt::{
    Client, ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder,
    DisconnectOptionsBuilder, Message, Properties, PropertyCode, ReasonCode, SslOptionsBuilder,
};

pub fn qos_list() -> Vec<i32> {
    vec![0, 1, 2]
}

pub fn protocol_versions() -> Vec<u32> {
    vec![3, 4, 5]
}

pub fn network_types() -> Vec<String> {
    vec![
        "tcp".to_string(),
        "ws".to_string(),
        "wss".to_string(),
        "ssl".to_string(),
    ]
}

pub fn broker_addr_by_type(network_type: &str) -> String {
    let net = network_type.to_string();
    if net == "tcp" {
        broker_addr()
    } else if net == "ws" {
        broker_ws_addr()
    } else if net == "wss" {
        broker_wss_addr()
    } else {
        broker_ssl_addr()
    }
}

pub fn ws_by_type(network_type: &str) -> bool {
    let net = network_type.to_string();
    net == "ws" || net == "wss"
}

pub fn ssl_by_type(network_type: &str) -> bool {
    let net = network_type.to_string();
    net == "ssl" || net == "wss"
}

pub fn build_conn_pros(
    client_test_properties: ClientTestProperties,
    err_pwd: bool,
) -> ConnectOptions {
    if client_test_properties.mqtt_version == 4 || client_test_properties.mqtt_version == 3 {
        build_v34_conn_pros(client_test_properties.clone(), err_pwd)
    } else {
        let mut props = build_v5_pros();
        if client_test_properties.request_response {
            props
                .push_val(PropertyCode::RequestResponseInformation, 1)
                .unwrap();
        }
        props
            .push_int(PropertyCode::MaximumPacketSize, 128)
            .unwrap();
        build_v5_conn_pros(client_test_properties, props, err_pwd)
    }
}

pub fn connect_server(client_properties: &ClientTestProperties) -> Client {
    let create_opts = build_create_conn_pros(&client_properties.client_id, &client_properties.addr);

    let cli_res = Client::new(create_opts);
    assert!(cli_res.is_ok());
    let cli = cli_res.unwrap();

    let conn_opts = build_conn_pros(client_properties.clone(), client_properties.err_pwd);
    let result = cli.connect(conn_opts);
    if client_properties.conn_is_err {
        assert!(result.is_err());
    } else {
        assert!(result.is_ok());
    }
    cli
}

pub fn publish_data(cli: &Client, message: Message, is_err: bool) {
    let err = cli.publish(message);
    println!("{:?}", err);
    if is_err {
        assert!(err.is_err());
    } else {
        assert!(err.is_ok());
    }
}

pub fn subscribe_data_by_qos<T>(cli: &Client, sub_topic: &str, sub_qos: i32, call_fn: T)
where
    T: Fn(Message) -> bool,
{
    let rx = cli.start_consuming();
    let res = cli.subscribe(sub_topic, sub_qos);
    assert!(res.is_ok());

    loop {
        let res = rx.recv_timeout(Duration::from_secs(10));
        println!("{:?}", res);
        assert!(res.is_ok());
        let msg_opt = res.unwrap();
        assert!(msg_opt.is_some());
        let msg = msg_opt.unwrap();
        if call_fn(msg) {
            break;
        }
    }
}

pub fn build_client_id(name: &str) -> String {
    format!("{}-{}", name, unique_id())
}

pub fn broker_addr() -> String {
    "tcp://127.0.0.1:1883".to_string()
}

pub fn broker_ssl_addr() -> String {
    "mqtts://127.0.0.1:8883".to_string()
}

pub fn broker_ws_addr() -> String {
    "ws://127.0.0.1:8093".to_string()
}

pub fn broker_wss_addr() -> String {
    "wss://127.0.0.1:8094".to_string()
}

pub fn broker_grpc_addr() -> String {
    "127.0.0.1:9981".to_string()
}

pub fn username() -> String {
    "admin".to_string()
}

pub fn password() -> String {
    "pwd123".to_string()
}

pub fn err_password() -> String {
    "pwd1235".to_string()
}

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

pub fn build_v5_conn_pros(
    client_test_properties: ClientTestProperties,
    props: Properties,
    err_pwd: bool,
) -> ConnectOptions {
    let pwd = if err_pwd { err_password() } else { password() };
    let mut conn_opts = if client_test_properties.ws {
        ConnectOptionsBuilder::new_ws_v5()
    } else {
        ConnectOptionsBuilder::new_v5()
    };
    if client_test_properties.ssl {
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

    let uname = if client_test_properties.user_name.is_empty() {
        username()
    } else {
        client_test_properties.user_name
    };

    let password = if client_test_properties.password.is_empty() {
        pwd
    } else {
        client_test_properties.password
    };

    conn_opts
        .keep_alive_interval(Duration::from_secs(600))
        .clean_start(true)
        .connect_timeout(Duration::from_secs(60))
        .properties(props.clone())
        .user_name(uname)
        .password(password);

    if client_test_properties.will.is_some() {
        conn_opts.will_message(client_test_properties.will.unwrap());
    }
    conn_opts.finalize()
}

pub fn build_v34_conn_pros(
    client_test_properties: ClientTestProperties,
    err_pwd: bool,
) -> ConnectOptions {
    let pwd = if err_pwd { err_password() } else { password() };
    let mut conn_opts =
        ConnectOptionsBuilder::with_mqtt_version(client_test_properties.mqtt_version);

    if client_test_properties.ssl {
        let ssl_opts = SslOptionsBuilder::new()
            .trust_store(get_cargo_manifest_dir())
            .unwrap()
            .verify(false)
            .disable_default_trust_store(false)
            .finalize();
        conn_opts.ssl_options(ssl_opts);
    }

    let uname = if client_test_properties.user_name.is_empty() {
        username()
    } else {
        client_test_properties.user_name
    };

    let password = if client_test_properties.password.is_empty() {
        pwd
    } else {
        client_test_properties.password
    };

    conn_opts
        .keep_alive_interval(Duration::from_secs(600))
        .clean_session(true)
        .connect_timeout(Duration::from_secs(50))
        .user_name(uname)
        .password(password)
        .finalize()
}

fn get_cargo_manifest_dir() -> String {
    format!(
        "{}/../config/example/certs/ca.pem",
        env!("CARGO_MANIFEST_DIR")
    )
}

pub fn build_create_conn_pros(client_id: &str, addr: &str) -> CreateOptions {
    if client_id.is_empty() {
        CreateOptionsBuilder::new().server_uri(addr).finalize()
    } else {
        CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id)
            .finalize()
    }
}

pub fn distinct_conn(cli: Client) {
    let mut props = Properties::new();

    props
        .push_string_pair(
            PropertyCode::UserProperty,
            "DISCONNECT_FLAG_NOT_DELETE_SESSION",
            "true",
        )
        .unwrap();

    let disconnect_opts = DisconnectOptionsBuilder::new()
        .reason_code(ReasonCode::DisconnectWithWillMessage)
        .properties(props)
        .finalize();
    let res = cli.disconnect(disconnect_opts);
    assert!(res.is_ok());
}

pub fn distinct_conn_close(cli: Client) {
    let mut props = Properties::new();

    props
        .push_string_pair(
            PropertyCode::UserProperty,
            "DISCONNECT_FLAG_NOT_DELETE_SESSION",
            "false",
        )
        .unwrap();

    let disconnect_opts = DisconnectOptionsBuilder::new()
        .reason_code(ReasonCode::DisconnectWithWillMessage)
        .properties(props)
        .finalize();
    let res = cli.disconnect(disconnect_opts);
    assert!(res.is_ok());
}

// #[allow(dead_code)]
// pub fn connect_server5_response_information(client_id: &str, addr: &str) -> (Client, String) {
//     let mqtt_version = 5;
//     let mut props = build_v5_pros();
//     props
//         .push_val(PropertyCode::RequestResponseInformation, 1)
//         .unwrap();

//     let create_opts = build_create_pros(client_id, addr);
//     let cli = Client::new(create_opts).unwrap_or_else(|err| {
//         println!("Error creating the client: {:?}", err);
//         process::exit(1);
//     });

//     let conn_opts = build_v5_conn_pros(props.clone(), false, false, false);
//     let response_information = match cli.connect(conn_opts) {
//         Ok(response) => {
//             let resp = response.connect_response().unwrap();

//             assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
//             assert_eq!(mqtt_version, resp.mqtt_version);
//             assert!(resp.session_present);
//             assert_eq!(response.reason_code(), ReasonCode::Success);

//             let resp_pros = response.properties();
//             resp_pros
//                 .get_string(PropertyCode::ResponseInformation)
//                 .unwrap()
//         }
//         Err(e) => {
//             println!("Unable to connect:\n\t{:?}", e);
//             process::exit(1);
//         }
//     };
//     (cli, response_information)
// }
