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
    use admin_server::mqtt::session::{SessionListReq, SessionListRow};
    use common_base::tools::{now_second, unique_id};
    use paho_mqtt::{Message, QOS_1};
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
            distinct_conn_close, publish_data, session_expiry_interval, ssl_by_type, ws_by_type,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn session_expire_test() {
        let network = "tcp";
        let qos = QOS_1;
        let client_id = build_client_id(format!("sub_auto_test_{network}_{qos}").as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let topic = format!("/tests/v1/v2/{}", unique_id());

        let message_content = "session expire test".to_string();
        let msg = Message::new(topic.clone(), message_content.clone(), QOS_1);
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        let admin_client = create_test_env().await;

        let check_fn = async {
            loop {
                let request = SessionListReq {
                    client_id: None,
                    limit: Some(10000),
                    page: Some(1),
                    sort_field: None,
                    sort_by: None,
                    filter_field: None,
                    filter_values: None,
                    exact_match: None,
                };
                let res = admin_client
                    .get_session_list::<SessionListReq, Vec<SessionListRow>>(&request)
                    .await;
                assert!(res.is_ok());
                let sessions = res.unwrap().data;
                if !contain_session(&sessions, &client_id) {
                    return;
                }
                sleep(Duration::from_secs(1)).await;
            }
        };

        let start_time = now_second();
        let res = timeout(Duration::from_secs(180), check_fn).await;
        assert!(res.is_ok());
        let total = now_second() - start_time;
        println!("total:{total}");
        let sei = session_expiry_interval() as u64;
        assert!(total >= (sei - 5) && total <= (sei + 5));
    }

    #[tokio::test]
    async fn session_close_test() {
        let network = "tcp";
        let qos = QOS_1;
        let client_id = build_client_id(format!("sub_auto_test_{network}_{qos}").as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        let topic = format!("/tests/v1/v2/{}", unique_id());

        let message_content = "session close test".to_string();
        let msg = Message::new(topic.clone(), message_content.clone(), QOS_1);
        publish_data(&cli, msg, false);
        distinct_conn_close(cli);

        let admin_client = create_test_env().await;

        let request = SessionListReq {
            client_id: None,
            limit: Some(10000),
            page: Some(1),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };
        let res = admin_client
            .get_session_list::<SessionListReq, Vec<SessionListRow>>(&request)
            .await;
        assert!(res.is_ok());
        let sessions = res.unwrap().data;
        assert!(!contain_session(&sessions, &client_id));
    }

    fn contain_session(sessions: &Vec<SessionListRow>, client_id: &str) -> bool {
        let mut flag = false;
        for session in sessions {
            if session.client_id == *client_id {
                flag = true;
            }
        }
        flag
    }
}
