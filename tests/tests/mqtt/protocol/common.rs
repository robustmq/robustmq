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

use std::time::{Duration, Instant};

use crate::mqtt::protocol::ClientTestProperties;
use admin_server::client::AdminHttpClient;
use admin_server::{
    mqtt::session::{SessionListReq, SessionListRow},
    tool::PageReplyData,
};
use common_base::tools::now_nanos;
use common_base::uuid::unique_id;
use paho_mqtt::{
    Client, ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder,
    DisconnectOptionsBuilder, Message, Properties, PropertyCode, ReasonCode, SslOptionsBuilder,
    SubscribeOptions,
};
use tokio::time::sleep;

pub fn qos_list() -> Vec<i32> {
    vec![0, 1, 2]
}

pub fn protocol_versions() -> Vec<u32> {
    vec![3, 4, 5]
}

pub async fn create_test_env() -> AdminHttpClient {
    AdminHttpClient::new("http://127.0.0.1:8080")
}

pub async fn session_list_by_admin(client_id: &str) -> PageReplyData<Vec<SessionListRow>> {
    let admin_client = create_test_env().await;
    let request = SessionListReq {
        client_id: Some(client_id.to_string()),
        ..Default::default()
    };
    admin_client.get_session_list(&request).await.unwrap()
}

pub async fn session_count_by_admin(client_id: &str) -> usize {
    session_list_by_admin(client_id).await.total_count
}

pub async fn wait_for_session_count_by_admin(
    client_id: &str,
    expected: usize,
    max_wait: Duration,
) -> Duration {
    let start = Instant::now();
    loop {
        let count = session_count_by_admin(client_id).await;
        if count == expected {
            return start.elapsed();
        }
        if start.elapsed() >= max_wait {
            panic!(
                "wait_for_session_count_by_admin timeout: client_id={client_id}, expected={expected}, last_count={count}"
            );
        }
        sleep(Duration::from_secs(1)).await;
    }
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

pub fn test_client_id() -> String {
    unique_id()
}

pub fn ssl_by_type(network_type: &str) -> bool {
    let net = network_type.to_string();
    net == "ssl" || net == "wss"
}

pub fn uniq_topic() -> String {
    format!("/{}/{}", unique_id(), now_nanos())
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
        if let Some(size) = client_test_properties.packet_size {
            props
                .push_int(PropertyCode::MaximumPacketSize, size as i32)
                .unwrap();
        } else {
            props
                .push_int(PropertyCode::MaximumPacketSize, 1024 * 1024)
                .unwrap();
        }
        build_v5_conn_pros(client_test_properties, props, err_pwd)
    }
}

pub fn connect_server(client_properties: &ClientTestProperties) -> Client {
    let create_opts = build_create_conn_pros(&client_properties.client_id, &client_properties.addr);
    let cli = Client::new(create_opts).unwrap();
    let conn_opts = build_conn_pros(client_properties.clone(), client_properties.err_pwd);
    let result = cli.connect(conn_opts);
    if result.is_err() {
        println!("result:{result:?}");
    }
    if client_properties.conn_is_err {
        assert!(result.is_err());
    } else {
        assert!(result.is_ok());
    }
    cli
}

pub fn new_client(addr: &str, client_id: &str) -> Client {
    let create_opts = build_create_conn_pros(client_id, addr);
    Client::new(create_opts).unwrap()
}

pub fn connect_mqtt34(addr: &str, client_id: &str, clean_session: bool) -> (Client, bool) {
    let cli = new_client(addr, client_id);
    let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(3);
    conn_opts
        .user_name(username())
        .password(password())
        .clean_session(clean_session);
    let result = cli.connect(conn_opts.finalize()).unwrap();
    assert_eq!(result.reason_code(), ReasonCode::Success);
    let session_present = result.connect_response().unwrap().session_present;
    (cli, session_present)
}

pub fn connect_mqtt5(
    addr: &str,
    client_id: &str,
    clean_start: bool,
    expiry: u32,
) -> (Client, bool) {
    let cli = new_client(addr, client_id);
    let mut conn_opts = ConnectOptionsBuilder::new_v5();

    let mut props = Properties::new();
    props
        .push_u32(PropertyCode::SessionExpiryInterval, expiry)
        .unwrap();

    conn_opts
        .user_name(username())
        .password(password())
        .properties(props)
        .clean_start(clean_start);

    let result = cli.connect(conn_opts.finalize()).unwrap();
    assert_eq!(result.reason_code(), ReasonCode::Success);
    let session_present = result.connect_response().unwrap().session_present;
    (cli, session_present)
}

pub fn publish_data(cli: &Client, message: Message, is_err: bool) {
    let err = cli.publish(message);

    if is_err {
        assert!(err.is_err());
    } else {
        assert!(err.is_ok());
    }
}

pub fn subscribe_data_by_qos<T>(
    cli: &Client,
    sub_topic: &str,
    sub_qos: i32,
    call_fn: T,
) -> Result<(), String>
where
    T: Fn(Message) -> bool,
{
    subscribe_data_by_qos_with_timeout(cli, sub_topic, sub_qos, Duration::from_secs(60), call_fn)
}

pub fn subscribe_data_by_qos_with_timeout<T>(
    cli: &Client,
    sub_topic: &str,
    sub_qos: i32,
    timeout: Duration,
    call_fn: T,
) -> Result<(), String>
where
    T: Fn(Message) -> bool,
{
    let rx = cli.start_consuming();
    let res = cli.subscribe(sub_topic, sub_qos);
    assert!(res.is_ok());

    let start = Instant::now();
    let poll_interval = Duration::from_secs(1);
    loop {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return Err(format!(
                "subscribe_data_by_qos timeout after {} seconds",
                timeout.as_secs()
            ));
        }

        let remaining = timeout.saturating_sub(elapsed);
        let wait_for = remaining.min(poll_interval);
        let res = rx.recv_timeout(wait_for);
        if let Ok(msg_opt) = res {
            assert!(msg_opt.is_some());
            let msg = msg_opt.unwrap();
            if call_fn(msg) {
                return Ok(());
            }
        }
    }
}

pub struct SubscribeTestData<S, T, P>
where
    S: Into<String>,
    T: Into<SubscribeOptions>,
    P: Into<Option<Properties>>,
{
    pub(crate) sub_topic: S,
    pub(crate) sub_qos: i32,
    pub(crate) subscribe_options: T,
    pub(crate) subscribe_properties: P,
}

pub async fn subscribe_data_with_options<S, T, P, F>(
    cli: &Client,
    subscribe_test_data: SubscribeTestData<S, T, P>,
    call_fn: F,
) -> Result<(), String>
where
    S: Into<String>,
    T: Into<SubscribeOptions>,
    P: Into<Option<Properties>>,
    F: Fn(Message) -> bool,
{
    let rx = cli.start_consuming();
    let res = cli.subscribe_with_options(
        subscribe_test_data.sub_topic.into(),
        subscribe_test_data.sub_qos,
        subscribe_test_data.subscribe_options.into(),
        subscribe_test_data.subscribe_properties.into(),
    );

    if let Err(e) = res {
        return Err(format!("Failed to subscribe: {:?}", e));
    }

    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(30);
    let poll_interval = Duration::from_secs(1);

    loop {
        let elapsed = start_time.elapsed();
        if elapsed >= timeout_duration {
            return Err(format!(
                "subscribe_data_with_options timeout after {} seconds",
                timeout_duration.as_secs()
            ));
        }

        let remaining = timeout_duration.saturating_sub(elapsed);
        let wait_for = remaining.min(poll_interval);
        let res = rx.recv_timeout(wait_for);
        if let Ok(Some(msg)) = res {
            if call_fn(msg) {
                return Ok(());
            }
        }
    }
}

pub fn build_client_id(_name: &str) -> String {
    // format!("{}_{}_{}", name, unique_id(), now_nanos())
    unique_id()
}

pub fn broker_addr() -> String {
    "tcp://localhost:1883".to_string()
}

pub fn broker_ssl_addr() -> String {
    "mqtts://localhost:1885".to_string()
}

pub fn broker_ws_addr() -> String {
    "ws://localhost:8083".to_string()
}

pub fn broker_wss_addr() -> String {
    "wss://localhost:8085".to_string()
}

pub fn broker_grpc_addr() -> String {
    "localhost:1228".to_string()
}

pub fn username() -> String {
    "admin".to_string()
}

pub fn password() -> String {
    "robustmq".to_string()
}

pub fn err_password() -> String {
    "pwd1235".to_string()
}

pub fn build_v5_pros() -> Properties {
    let mut props = Properties::new();
    props
        .push_u32(
            PropertyCode::SessionExpiryInterval,
            session_expiry_interval(),
        )
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
        let ssl_path = get_cargo_manifest_dir();
        let ssl_opts = SslOptionsBuilder::new()
            .trust_store(ssl_path)
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
        .keep_alive_interval(Duration::from_secs(kee_alive_interval()))
        .clean_start(true)
        .connect_timeout(Duration::from_secs(60))
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(5))
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
        .keep_alive_interval(Duration::from_secs(kee_alive_interval()))
        .clean_session(true)
        .connect_timeout(Duration::from_secs(50))
        .user_name(uname)
        .password(password)
        .finalize()
}

fn get_cargo_manifest_dir() -> String {
    format!("{}/../config/certs/ca.pem", env!("CARGO_MANIFEST_DIR"))
}

pub fn kee_alive_interval() -> u64 {
    60
}

pub fn session_expiry_interval() -> u32 {
    30
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
    let props = Properties::new();
    let disconnect_opts = DisconnectOptionsBuilder::new()
        .reason_code(ReasonCode::DisconnectWithWillMessage)
        .properties(props)
        .finalize();
    let _ = cli.disconnect(disconnect_opts);
}
