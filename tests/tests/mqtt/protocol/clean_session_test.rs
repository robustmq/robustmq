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
        broker_addr_by_type, connect_mqtt34, connect_mqtt5, distinct_conn, session_count_by_admin,
        session_list_by_admin, test_client_id,
    };
    use admin_server::tool::PageReplyData;
    use common_base::uuid::unique_id;
    use paho_mqtt::{DisconnectOptionsBuilder, Properties, PropertyCode, ReasonCode};
    use std::time::Duration;
    use tokio::time::sleep;

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
            sleep(Duration::from_secs(1)).await;
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
        let data: PageReplyData<Vec<admin_server::mqtt::session::SessionListRow>> =
            session_list_by_admin(&client_id).await;
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
        let data = session_list_by_admin(&client_id).await;
        assert_eq!(session_count_by_admin(&client_id).await, 1);
        let raw = data.data.first().unwrap();
        assert_eq!(raw.session_expiry, 60);
    }
}
