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
    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, build_conn_pros, build_create_conn_pros,
            distinct_conn, kee_alive_interval, network_types, session_expiry_interval, ssl_by_type,
            ws_by_type,
        },
        ClientTestProperties,
    };
    use mqtt_broker::core::connection::REQUEST_RESPONSE_PREFIX_NAME;
    use paho_mqtt::{Client, PropertyCode, ReasonCode};
    #[tokio::test]
    async fn response_properties_check_test() {
        for network in network_types() {
            let addr = broker_addr_by_type(&network);
            let client_id =
                build_client_id(format!("response_properties_check_test_{network}").as_str());
            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id,
                addr,
                ws: ws_by_type(&network),
                ssl: ssl_by_type(&network),
                ..Default::default()
            };
            let create_opts =
                build_create_conn_pros(&client_properties.client_id, &client_properties.addr);

            let cli_res = Client::new(create_opts);
            assert!(cli_res.is_ok());
            let cli = cli_res.unwrap();

            let conn_opts = build_conn_pros(client_properties.clone(), false);
            let result = cli.connect(conn_opts);
            println!("{result:?}");
            assert!(result.is_ok());
            let response = result.unwrap();
            assert_eq!(response.reason_code(), ReasonCode::Success);

            let resp_pros = response.properties();
            println!("{resp_pros:?}");
            assert_eq!(
                resp_pros
                    .get(PropertyCode::SessionExpiryInterval)
                    .unwrap()
                    .get_int()
                    .unwrap(),
                session_expiry_interval() as i32
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

            assert!(resp_pros
                .get(PropertyCode::AssignedClientIdentifer)
                .is_none());

            assert_eq!(
                resp_pros
                    .get(PropertyCode::TopicAliasMaximum)
                    .unwrap()
                    .get_int()
                    .unwrap(),
                65535
            );

            assert!(resp_pros.get(PropertyCode::ReasonString).is_none());

            assert!(resp_pros.get(PropertyCode::UserProperty).is_none());

            assert_eq!(
                resp_pros
                    .get(PropertyCode::WildcardSubscriptionAvailable)
                    .unwrap()
                    .get_int()
                    .unwrap(),
                1
            );

            assert_eq!(
                resp_pros
                    .get(PropertyCode::SubscriptionIdentifiersAvailable)
                    .unwrap()
                    .get_int()
                    .unwrap(),
                1
            );

            assert_eq!(
                resp_pros
                    .get(PropertyCode::SharedSubscriptionAvailable)
                    .unwrap()
                    .get_int()
                    .unwrap(),
                1
            );

            assert_eq!(
                resp_pros
                    .get(PropertyCode::ServerKeepAlive)
                    .unwrap()
                    .get_int()
                    .unwrap(),
                kee_alive_interval() as i32
            );

            assert!(resp_pros.get(PropertyCode::ResponseInformation).is_none());
            assert!(resp_pros.get(PropertyCode::ServerReference).is_none());
            assert!(resp_pros.get(PropertyCode::AuthenticationMethod).is_none());
            assert!(resp_pros.get(PropertyCode::AuthenticationData).is_none());

            distinct_conn(cli);
        }
    }

    #[tokio::test]
    async fn request_response_test() {
        for network in network_types() {
            let addr = broker_addr_by_type(&network);
            let client_id = build_client_id(format!("request_response_test_{network}").as_str());
            let client_properties = ClientTestProperties {
                mqtt_version: 5,
                client_id,
                addr,
                ws: ws_by_type(&network),
                ssl: ssl_by_type(&network),
                request_response: true,
                ..Default::default()
            };
            let create_opts =
                build_create_conn_pros(&client_properties.client_id, &client_properties.addr);

            let cli_res = Client::new(create_opts);
            assert!(cli_res.is_ok());
            let cli = cli_res.unwrap();
            println!("{client_properties:?}");

            let conn_opts = build_conn_pros(client_properties.clone(), false);
            let result = cli.connect(conn_opts);
            println!("{result:?}");
            assert!(result.is_ok());
            let response = result.unwrap();
            assert_eq!(response.reason_code(), ReasonCode::Success);

            let resp_pros = response.properties();
            println!("{resp_pros:?}");
            assert_eq!(
                resp_pros
                    .get(PropertyCode::ResponseInformation)
                    .unwrap()
                    .get_string()
                    .unwrap(),
                REQUEST_RESPONSE_PREFIX_NAME.to_string()
            );

            distinct_conn(cli);
        }
    }
}
