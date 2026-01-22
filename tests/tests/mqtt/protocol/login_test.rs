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
    use crate::mqtt::protocol::common::{
        broker_addr_by_type, build_client_id, build_conn_pros, build_create_conn_pros,
        network_types, protocol_versions, ssl_by_type, ws_by_type,
    };
    use crate::mqtt::protocol::ClientTestProperties;
    use paho_mqtt::Client;

    #[tokio::test]
    async fn client_connect_wrong_password_test() {
        for protocol_ver in protocol_versions() {
            for network in network_types() {
                let client_id = build_client_id(
                    format!("client_connect_wrong_password_test_{protocol_ver}_{network}").as_str(),
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
                password_test(client_properties, true);
            }
        }
    }

    #[tokio::test]
    async fn client_connect_right_password_test() {
        for protocol_ver in protocol_versions() {
            for network in network_types() {
                let client_id = build_client_id(
                    format!("client_connect_test_{protocol_ver}_{network}").as_str(),
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
                password_test(client_properties, false);
            }
        }
    }

    fn password_test(client_properties: ClientTestProperties, is_err: bool) {
        let create_opts =
            build_create_conn_pros(&client_properties.client_id, &client_properties.addr);

        let cli_res = Client::new(create_opts);
        assert!(cli_res.is_ok());
        let cli = cli_res.unwrap();

        let conn_opts = build_conn_pros(client_properties.clone(), is_err);
        let result = cli.connect(conn_opts);
        if is_err {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }
    }
}
