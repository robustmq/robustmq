mod common;

#[cfg(test)]
mod tests {
    use crate::common::{broker_addr, err_password, password, username};
    use common_base::tools::unique_id;
    use paho_mqtt::{
        Client, ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder,
        DisconnectOptionsBuilder, Properties, PropertyCode, ReasonCode,
    };
    use std::{process, time::Duration};

    #[tokio::test]
    async fn client34_connect_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr);
        v3_session_present_test(mqtt_version, &client_id, &addr);

        let mqtt_version = 4;
        let client_id = unique_id();
        let addr = broker_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr);
        v3_session_present_test(mqtt_version, &client_id, &addr);
    }

    #[tokio::test]
    async fn client5_connect_test() {
        let client_id = unique_id();
        let addr = broker_addr();
        v5_wrong_password_test(&client_id, &addr);
        v5_session_present_test(&client_id, &addr);
        v5_response_test(&client_id, &addr);
        v5_assigned_client_id_test(&addr);
    }

    fn v3_wrong_password_test(mqtt_version: u32, client_id: &String, addr: &String) {
        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v3_conn_pros(mqtt_version, true);

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

    fn v3_session_present_test(mqtt_version: u32, client_id: &String, addr: &String) {
        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v3_conn_pros(mqtt_version, false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);

        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v3_conn_pros(mqtt_version, false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(!resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }

        distinct_conn(cli);
    }

    fn v5_wrong_password_test(client_id: &String, addr: &String) {
        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let props = build_v5_pros();
        let conn_opts = build_v5_conn_pros(props, true);
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

    fn v5_session_present_test(client_id: &String, addr: &String) {
        let mqtt_version = 5;
        let props = build_v5_pros();

        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(props.clone(), false);
        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);

        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(props.clone(), false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(!resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }

    fn v5_assigned_client_id_test(addr: &String) {
        let mqtt_version = 5;
        let client_id = "".to_string();
        let props = build_v5_pros();

        let create_opts = build_create_pros(&client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(props.clone(), false);
        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(resp.session_present);
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

    fn v5_response_test(client_id: &String, addr: &String) {
        let mqtt_version = 5;

        let pros = build_v5_pros();
        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(pros.clone(), false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                // response
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(!resp.session_present);
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
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }

    fn build_v5_pros() -> Properties {
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

    fn build_v5_conn_pros(props: Properties, err_pwd: bool) -> ConnectOptions {
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

    fn build_v3_conn_pros(mqtt_version: u32, err_pwd: bool) -> ConnectOptions {
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

    fn build_create_pros(client_id: &String, addr: &String) -> CreateOptions {
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

    fn distinct_conn(cli: Client) {
        let disconnect_opts = DisconnectOptionsBuilder::new()
            .reason_code(ReasonCode::DisconnectWithWillMessage)
            .finalize();
        cli.disconnect(disconnect_opts).unwrap();
    }
}
