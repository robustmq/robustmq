use paho_mqtt::{
    Client, ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder,
    DisconnectOptionsBuilder, Properties, PropertyCode, ReasonCode,
};
use std::time::Duration;

pub fn broker_addr() -> String {
    return "tcp://127.0.0.1:1883".to_string();
}

pub fn username() -> String {
    return "admin".to_string();
}

pub fn password() -> String {
    return "pwd123".to_string();
}
pub fn err_password() -> String {
    return "pwd1235".to_string();
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
        .push_val(PropertyCode::RequestResponseInformation, 1)
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
    return props;
}

pub fn build_v5_conn_pros(props: Properties, err_pwd: bool) -> ConnectOptions {
    let pwd = if err_pwd { err_password() } else { password() };
    let conn_opts = ConnectOptionsBuilder::new_v5()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_start(true)
        .connect_timeout(Duration::from_secs(1))
        .properties(props.clone())
        .user_name(username())
        .password(pwd)
        .finalize();
    return conn_opts;
}

pub fn build_v3_conn_pros(mqtt_version: u32, err_pwd: bool) -> ConnectOptions {
    let pwd = if err_pwd { err_password() } else { password() };
    let conn_opts = ConnectOptionsBuilder::with_mqtt_version(mqtt_version)
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .connect_timeout(Duration::from_secs(1))
        .user_name(username())
        .password(pwd)
        .finalize();
    return conn_opts;
}

pub fn build_create_pros(client_id: &String, addr: &String) -> CreateOptions {
    let create_opts = if client_id.is_empty() {
        CreateOptionsBuilder::new()
            .server_uri(addr.clone())
            .finalize()
    } else {
        CreateOptionsBuilder::new()
            .server_uri(addr.clone())
            .client_id(client_id.clone())
            .finalize()
    };
    return create_opts;
}

pub fn distinct_conn(cli: Client) {
    let disconnect_opts = DisconnectOptionsBuilder::new()
        .reason_code(ReasonCode::DisconnectWithWillMessage)
        .finalize();
    cli.disconnect(disconnect_opts).unwrap();
}
