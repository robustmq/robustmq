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
    use std::sync::Arc;

    use grpc_clients::placement::mqtt::call::{
        placement_create_session, placement_delete_session, placement_list_session,
        placement_update_session,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::session::MqttSession;
    use protocol::placement_center::placement_center_mqtt::{
        CreateSessionRequest, DeleteSessionRequest, ListSessionRequest, UpdateSessionRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]
    async fn mqtt_session_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let cluster_name: String = "test_cluster".to_string();
        let client_id: String = "test_cient_id".to_string();
        let connection_id: u64 = 1;
        let broker_id: u64 = 1;
        let update_broker_id: u64 = 2;
        let session_expiry: u64 = 10000;
        let last_will_delay_interval: u64 = 10000;

        let mut mqtt_session: MqttSession = MqttSession::new(
            client_id.clone(),
            session_expiry,
            true,
            Some(last_will_delay_interval),
        );
        mqtt_session.update_broker_id(Some(broker_id));
        mqtt_session.update_connnction_id(Some(connection_id));

        let request = CreateSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: client_id.clone(),
            session: MqttSession::encode(&mqtt_session),
        };

        match placement_create_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = ListSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
        };

        match placement_list_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.sessions {
                    let session = serde_json::from_slice::<MqttSession>(raw.as_slice()).unwrap();
                    if mqtt_session == session {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        mqtt_session.update_broker_id(Some(update_broker_id));
        mqtt_session.update_reconnect_time();
        mqtt_session.update_distinct_time();

        let request = UpdateSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
            connection_id: mqtt_session.connection_id.unwrap(),
            broker_id: mqtt_session.broker_id.unwrap_or(1100),
            reconnect_time: mqtt_session.reconnect_time.unwrap(),
            distinct_time: mqtt_session.distinct_time.unwrap(),
        };

        match placement_update_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = ListSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
        };

        match placement_list_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.sessions {
                    let session = serde_json::from_slice::<MqttSession>(raw.as_slice()).unwrap();
                    if mqtt_session == session {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = DeleteSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
        };

        match placement_delete_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = ListSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
        };

        match placement_list_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag: bool = false;
                for raw in data.sessions {
                    let session = serde_json::from_slice::<MqttSession>(raw.as_slice()).unwrap();
                    if mqtt_session == session {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
