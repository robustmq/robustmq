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
    use crate::common::get_placement_addr;
    use common_base::tools::unique_id;
    use grpc_clients::meta::mqtt::call::{
        placement_create_session, placement_delete_session, placement_list_session,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::session::MqttSession;
    use protocol::meta::meta_service_mqtt::{
        CreateSessionRequest, DeleteSessionRequest, ListSessionRequest,
    };
    use std::sync::Arc;

    #[tokio::test]

    async fn mqtt_session_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let client_id: String = unique_id();
        let connection_id: u64 = 1;
        let broker_id: u64 = 1;
        let session_expiry: u64 = 10000;
        let last_will_delay_interval: u64 = 10000;

        let mut mqtt_session: MqttSession = MqttSession::new(
            client_id.clone(),
            session_expiry,
            true,
            Some(last_will_delay_interval),
        );
        mqtt_session.update_broker_id(Some(broker_id));
        mqtt_session.update_connection_id(Some(connection_id));

        let request = CreateSessionRequest {
            client_id: client_id.clone(),
            session: mqtt_session.encode().unwrap(),
        };

        placement_create_session(&client_pool, &addrs, request)
            .await
            .unwrap();

        let request = ListSessionRequest {
            client_id: mqtt_session.client_id.clone(),
        };

        let data = placement_list_session(&client_pool, &addrs, request)
            .await
            .unwrap();
        let mut flag: bool = false;
        for raw in data.sessions {
            let session = MqttSession::decode(&raw).unwrap();
            if mqtt_session == session {
                flag = true;
            }
        }
        assert!(flag);

        let request = ListSessionRequest {
            client_id: mqtt_session.client_id.clone(),
        };

        let data = placement_list_session(&client_pool, &addrs, request)
            .await
            .unwrap();
        let mut flag: bool = false;
        for raw in data.sessions {
            let session = MqttSession::decode(&raw).unwrap();
            if mqtt_session == session {
                flag = true;
            }
        }
        assert!(flag);

        let request = DeleteSessionRequest {
            client_id: mqtt_session.client_id.clone(),
        };

        placement_delete_session(&client_pool, &addrs, request)
            .await
            .unwrap();

        let request = ListSessionRequest {
            client_id: mqtt_session.client_id.clone(),
        };

        let data = placement_list_session(&client_pool, &addrs, request)
            .await
            .unwrap();

        let mut flag: bool = false;
        for raw in data.sessions {
            let session = MqttSession::decode(&raw).unwrap();
            if mqtt_session == session {
                flag = true;
            }
        }
        assert!(!flag);
    }
}
