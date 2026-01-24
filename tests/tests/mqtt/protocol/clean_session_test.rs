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
        broker_addr_by_type, create_test_env, distinct_conn, password, test_client_id, username,
    };
    use admin_server::{
        mqtt::session::{SessionListReq, SessionListRow},
        tool::PageReplyData,
    };
    use common_base::tools::unique_id;
    use paho_mqtt::{
        Client, ConnectOptionsBuilder, CreateOptionsBuilder, Properties, PropertyCode, ReasonCode,
    };
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn mqtt34_clean_session_true_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = test_client_id();
        for i in 0..3 {
            let create_opts = CreateOptionsBuilder::new()
                .server_uri(addr.clone())
                .client_id(client_id.clone())
                .finalize();

            let cli = Client::new(create_opts).unwrap();
            let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(3);
            conn_opts
                .user_name(username())
                .password(password())
                .clean_session(true);
            let conn_opts = conn_opts.finalize();

            let result = cli.connect(conn_opts).unwrap();
            assert_eq!(result.reason_code(), ReasonCode::Success);
            let conn_resp = result.connect_response().unwrap();
            assert!(!conn_resp.session_present);
            distinct_conn(cli);
            sleep(Duration::from_secs(1)).await;

            let admin_client = create_test_env().await;
            let request = SessionListReq {
                client_id: Some(client_id.clone()),
                ..Default::default()
            };
            let data: PageReplyData<Vec<SessionListRow>> =
                admin_client.get_session_list(&request).await.unwrap();
            println!("{},{},{:?}", i, client_id, data);
            assert_eq!(data.total_count, 0);
        }
    }

    #[tokio::test]
    async fn mqtt34_clean_session_false_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = test_client_id();
        for i in 0..3 {
            let create_opts = CreateOptionsBuilder::new()
                .server_uri(addr.clone())
                .client_id(client_id.clone())
                .finalize();

            let cli = Client::new(create_opts).unwrap();
            let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(3);
            conn_opts
                .user_name(username())
                .password(password())
                .clean_session(false);
            let conn_opts = conn_opts.finalize();

            let result = cli.connect(conn_opts).unwrap();
            assert_eq!(result.reason_code(), ReasonCode::Success);
            if i == 0 {
                let conn_resp = result.connect_response().unwrap();
                assert!(!conn_resp.session_present);
                distinct_conn(cli);
                sleep(Duration::from_secs(1)).await;

                let admin_client = create_test_env().await;
                let request = SessionListReq {
                    client_id: Some(client_id.clone()),
                    ..Default::default()
                };
                let data: PageReplyData<Vec<SessionListRow>> =
                    admin_client.get_session_list(&request).await.unwrap();
                assert_eq!(data.total_count, 1);
            } else {
                let conn_resp = result.connect_response().unwrap();
                assert!(conn_resp.session_present);
                distinct_conn(cli);
                sleep(Duration::from_secs(1)).await;

                let admin_client = create_test_env().await;

                let request = SessionListReq {
                    client_id: Some(client_id.clone()),
                    ..Default::default()
                };
                let data: PageReplyData<Vec<SessionListRow>> =
                    admin_client.get_session_list(&request).await.unwrap();
                assert_eq!(data.total_count, 1);
            }
        }
    }

    #[tokio::test]
    async fn mqtt5_clean_session_true_expiry_0_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = test_client_id();
        for _ in 0..3 {
            let create_opts = CreateOptionsBuilder::new()
                .server_uri(addr.clone())
                .client_id(client_id.clone())
                .finalize();

            let cli = Client::new(create_opts).unwrap();
            let mut conn_opts = ConnectOptionsBuilder::new_v5();
            let mut props = Properties::new();
            props
                .push_u32(PropertyCode::SessionExpiryInterval, 0)
                .unwrap();
            conn_opts
                .user_name(username())
                .password(password())
                .properties(props)
                .clean_start(true);
            let conn_opts = conn_opts.finalize();

            let result = cli.connect(conn_opts).unwrap();
            assert_eq!(result.reason_code(), ReasonCode::Success);
            let conn_resp = result.connect_response().unwrap();
            assert!(!conn_resp.session_present);
            distinct_conn(cli);

            sleep(Duration::from_secs(1)).await;
            let admin_client = create_test_env().await;
            let request = SessionListReq {
                client_id: Some(client_id.clone()),
                ..Default::default()
            };
            let data: PageReplyData<Vec<SessionListRow>> =
                admin_client.get_session_list(&request).await.unwrap();
            assert_eq!(data.total_count, 0);
        }
    }

    #[tokio::test]
    async fn mqtt5_clean_session_true_expiry_30_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        for _ in 0..3 {
            let create_opts = CreateOptionsBuilder::new()
                .server_uri(addr.clone())
                .client_id(client_id.clone())
                .finalize();

            let cli = Client::new(create_opts).unwrap();
            let mut conn_opts = ConnectOptionsBuilder::new_v5();
            let mut props = Properties::new();
            props
                .push_u32(PropertyCode::SessionExpiryInterval, 30)
                .unwrap();
            conn_opts
                .user_name(username())
                .password(password())
                .properties(props)
                .clean_start(true);
            let conn_opts = conn_opts.finalize();

            let result = cli.connect(conn_opts).unwrap();
            assert_eq!(result.reason_code(), ReasonCode::Success);
            let conn_resp = result.connect_response().unwrap();
            assert!(!conn_resp.session_present);
            distinct_conn(cli);

            sleep(Duration::from_secs(1)).await;
            let admin_client = create_test_env().await;
            let request = SessionListReq {
                client_id: Some(client_id.clone()),
                ..Default::default()
            };
            let data: PageReplyData<Vec<SessionListRow>> =
                admin_client.get_session_list(&request).await.unwrap();
            assert_eq!(data.total_count, 1);
        }
    }

    #[tokio::test]
    async fn mqtt5_clean_session_false_expiry_0_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        for _ in 0..3 {
            let create_opts = CreateOptionsBuilder::new()
                .server_uri(addr.clone())
                .client_id(client_id.clone())
                .finalize();

            let cli = Client::new(create_opts).unwrap();
            let mut conn_opts = ConnectOptionsBuilder::new_v5();
            let mut props = Properties::new();
            props
                .push_u32(PropertyCode::SessionExpiryInterval, 0)
                .unwrap();
            conn_opts
                .user_name(username())
                .password(password())
                .properties(props)
                .clean_start(false);
            let conn_opts = conn_opts.finalize();

            let result = cli.connect(conn_opts).unwrap();
            assert_eq!(result.reason_code(), ReasonCode::Success);
            let conn_resp = result.connect_response().unwrap();
            assert!(!conn_resp.session_present);
            distinct_conn(cli);

            sleep(Duration::from_secs(1)).await;
            let admin_client = create_test_env().await;
            let request = SessionListReq {
                client_id: Some(client_id.clone()),
                ..Default::default()
            };
            let data: PageReplyData<Vec<SessionListRow>> =
                admin_client.get_session_list(&request).await.unwrap();
            assert_eq!(data.total_count, 0);
        }
    }

    #[tokio::test]
    async fn mqtt5_clean_session_false_expiry_30_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        for i in 0..3 {
            let create_opts = CreateOptionsBuilder::new()
                .server_uri(addr.clone())
                .client_id(client_id.clone())
                .finalize();

            let cli = Client::new(create_opts).unwrap();
            let mut conn_opts = ConnectOptionsBuilder::new_v5();
            let mut props = Properties::new();
            props
                .push_u32(PropertyCode::SessionExpiryInterval, 30)
                .unwrap();
            conn_opts
                .user_name(username())
                .password(password())
                .properties(props)
                .clean_start(false);
            let conn_opts = conn_opts.finalize();

            let result = cli.connect(conn_opts).unwrap();
            assert_eq!(result.reason_code(), ReasonCode::Success);
            let conn_resp = result.connect_response().unwrap();
            if i == 0 {
                assert!(!conn_resp.session_present);
                distinct_conn(cli);

                sleep(Duration::from_secs(1)).await;
                let admin_client = create_test_env().await;
                let request = SessionListReq {
                    client_id: Some(client_id.clone()),
                    ..Default::default()
                };
                let data: PageReplyData<Vec<SessionListRow>> =
                    admin_client.get_session_list(&request).await.unwrap();
                assert_eq!(data.total_count, 1);
            } else {
                assert!(conn_resp.session_present);
                distinct_conn(cli);

                sleep(Duration::from_secs(1)).await;
                let admin_client = create_test_env().await;
                let request = SessionListReq {
                    client_id: Some(client_id.clone()),
                    ..Default::default()
                };
                let data: PageReplyData<Vec<SessionListRow>> =
                    admin_client.get_session_list(&request).await.unwrap();
                assert_eq!(data.total_count, 1);
            }
        }
    }
}
