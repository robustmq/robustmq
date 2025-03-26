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

#[cfg(test)]
mod tests {
    use crate::mqtt_protocol::common::{
        broker_addr, broker_addr_by_type, broker_ssl_addr, broker_ws_addr, broker_wss_addr,
        build_client_id, build_conn_pros, build_create_conn_pros, distinct_conn, network_types,
        protocol_versions, ssl_by_type, ws_by_type,
    };
    use crate::mqtt_protocol::ClientTestProperties;
    use paho_mqtt::{Client, ReasonCode};

    #[tokio::test]
    async fn client_connect_wrong_password_test() {
        for protocol_ver in protocol_versions() {
            for network in network_types() {
                let client_id = build_client_id(
                    format!(
                        "client_connect_wrong_password_test_{}_{}",
                        protocol_ver, network
                    )
                    .as_str(),
                );
                let addr = broker_addr_by_type(&network);
                let client_properties = ClientTestProperties {
                    mqtt_version: protocol_ver,
                    client_id,
                    addr,
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };
                wrong_password_test(client_properties);
            }
        }
    }

    #[tokio::test]
    async fn client_connect_session_present_test() {
        for protocol_ver in protocol_versions() {
            for network in network_types() {
                let client_id = build_client_id(
                    format!(
                        "client_connect_session_present_test_{}_{}",
                        protocol_ver, network
                    )
                    .as_str(),
                );
                let addr = broker_addr_by_type(&network);
                let client_properties = ClientTestProperties {
                    mqtt_version: protocol_ver,
                    client_id,
                    addr,
                    ws: ws_by_type(&network),
                    ssl: ssl_by_type(&network),
                    ..Default::default()
                };

                create_session_connection(&client_properties, true);
                create_session_connection(&client_properties, false);
            }
        }
    }

    fn wrong_password_test(client_properties: ClientTestProperties) {
        let create_opts =
            build_create_conn_pros(&client_properties.client_id, &client_properties.addr);

        let cli_res = Client::new(create_opts);
        assert!(cli_res.is_ok());
        let cli = cli_res.unwrap();

        let conn_opts = build_conn_pros(client_properties.clone(), true);
        let result = cli.connect(conn_opts);
        println!(
            "client_test_properties:{:?},result:{:?}",
            client_properties, result
        );
        assert!(result.is_err());
    }

    fn create_session_connection(client_properties: &ClientTestProperties, present: bool) {
        let create_opts =
            build_create_conn_pros(&client_properties.client_id, &client_properties.addr);
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
        } else if client_properties.ssl {
            assert_eq!(format!("mqtts://{}", resp.server_uri), broker_ssl_addr());
        } else {
            assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
        }

        println!("client_properties:{:?},resp:{:?}", client_properties, resp);

        if present {
            assert!(resp.session_present);
        } else {
            assert!(!resp.session_present);
        }
        assert_eq!(client_properties.mqtt_version, resp.mqtt_version);
        assert_eq!(response.reason_code(), ReasonCode::Success);

        distinct_conn(cli);
    }
}
