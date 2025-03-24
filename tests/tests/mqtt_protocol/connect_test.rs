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
        broker_addr_by_type, build_client_id, network_types, protocol_versions, ssl_by_type,
        ws_by_type,
    };
    use crate::mqtt_protocol::connect_suite::wrong_password_test;
    use crate::mqtt_protocol::connect_suite::{create_session_connection, ClientTestProperties};

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
}
