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
        Client, ConnectOptionsBuilder, CreateOptionsBuilder, DisconnectOptionsBuilder, Properties,
        PropertyCode, ReasonCode,
    };
    use std::time::Duration;
    use tokio::time::sleep;

    async fn session_count_by_admin(client_id: &str) -> usize {
        let admin_client = create_test_env().await;
        let request = SessionListReq {
            client_id: Some(client_id.to_string()),
            ..Default::default()
        };
        let data: PageReplyData<Vec<SessionListRow>> =
            admin_client.get_session_list(&request).await.unwrap();
        data.total_count
    }

    fn new_client(addr: &str, client_id: &str) -> Client {
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id)
            .finalize();
        Client::new(create_opts).unwrap()
    }

    fn connect_mqtt34(addr: &str, client_id: &str, clean_session: bool) -> (Client, bool) {
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

    fn connect_mqtt5(
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

    // mqtt 3/4. clean_session true/false
    #[tokio::test]
    async fn mqtt34_clean_session_true_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = test_client_id();
        for _ in 0..3 {
            let (cli, session_present) = connect_mqtt34(&addr, &client_id, true);
            assert!(!session_present);
            distinct_conn(cli);
            sleep(Duration::from_secs(1)).await;
            assert_eq!(session_count_by_admin(&client_id).await, 0);
        }
    }

    #[tokio::test]
    async fn mqtt34_clean_session_false_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = test_client_id();
        for i in 0..3 {
            let (cli, session_present) = connect_mqtt34(&addr, &client_id, false);
            if i == 0 {
                assert!(!session_present);
                distinct_conn(cli);
                sleep(Duration::from_secs(1)).await;
                assert_eq!(session_count_by_admin(&client_id).await, 1);
            } else {
                assert!(session_present);
                distinct_conn(cli);
                sleep(Duration::from_secs(1)).await;
                assert_eq!(session_count_by_admin(&client_id).await, 1);
            }
        }
    }

    // mqtt 5 clean_session true/false. expiry 0/30
    #[tokio::test]
    async fn mqtt5_clean_session_true_expiry_0_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = test_client_id();
        for _ in 0..3 {
            let (cli, session_present) = connect_mqtt5(&addr, &client_id, true, 0);
            assert!(!session_present);
            distinct_conn(cli);

            sleep(Duration::from_secs(1)).await;
            assert_eq!(session_count_by_admin(&client_id).await, 0);
        }
    }

    #[tokio::test]
    async fn mqtt5_clean_session_true_expiry_30_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        for _ in 0..3 {
            let (cli, session_present) = connect_mqtt5(&addr, &client_id, true, 30);
            assert!(!session_present);
            distinct_conn(cli);

            sleep(Duration::from_secs(1)).await;
            assert_eq!(session_count_by_admin(&client_id).await, 1);
        }
    }

    #[tokio::test]
    async fn mqtt5_clean_session_false_expiry_0_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        for _ in 0..3 {
            let (cli, session_present) = connect_mqtt5(&addr, &client_id, false, 0);
            assert!(!session_present);
            distinct_conn(cli);

            sleep(Duration::from_secs(1)).await;
            assert_eq!(session_count_by_admin(&client_id).await, 0);
        }
    }

    #[tokio::test]
    async fn mqtt5_clean_distinct_expiry_60() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();

        let (cli, session_present) = connect_mqtt5(&addr, &client_id, false, 30);
        assert!(!session_present);

        // session expire = 30
        let admin_client = create_test_env().await;
        let request = SessionListReq {
            client_id: Some(client_id.to_string()),
            ..Default::default()
        };
        let data: PageReplyData<Vec<SessionListRow>> =
            admin_client.get_session_list(&request).await.unwrap();
        let raw = data.data.first().unwrap();
        assert_eq!(raw.session_expiry, 30);

        // distinct connection
        let mut props = Properties::new();
        props
            .push_u32(PropertyCode::SessionExpiryInterval, 60)
            .unwrap();
        let disconnect_opts = DisconnectOptionsBuilder::new()
            .reason_code(ReasonCode::DisconnectWithWillMessage)
            .properties(props)
            .finalize();
        let res = cli.disconnect(disconnect_opts);
        assert!(res.is_ok());

        sleep(Duration::from_secs(1)).await;

        // session expire = 30
        let request = SessionListReq {
            client_id: Some(client_id.to_string()),
            ..Default::default()
        };
        let data: PageReplyData<Vec<SessionListRow>> =
            admin_client.get_session_list(&request).await.unwrap();
        assert_eq!(session_count_by_admin(&client_id).await, 1);
        let raw = data.data.first().unwrap();
        assert_eq!(raw.session_expiry, 60);
    }
}
