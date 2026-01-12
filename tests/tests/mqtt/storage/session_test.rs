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
    use common_base::tools::now_second;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::session::MqttSession;
    use mqtt_broker::storage::session::SessionStorage;
    use std::sync::Arc;

    #[tokio::test]
    async fn session_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let session_storage = SessionStorage::new(client_pool);
        let client_id: String = "client_id_11111".to_string();
        let session = MqttSession {
            client_id: client_id.clone(),
            session_expiry: 1000,
            broker_id: Some(1),
            reconnect_time: Some(now_second()),
            ..Default::default()
        };

        session_storage
            .set_session(client_id.clone(), &session)
            .await
            .unwrap();

        let result = session_storage
            .get_session(client_id.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.client_id, client_id);
        assert_eq!(result.broker_id.unwrap(), 1);
        assert!(result.connection_id.is_none());

        session_storage
            .update_session(client_id.clone(), 3, 3, now_second(), 0)
            .await
            .unwrap();

        let result = session_storage
            .get_session(client_id.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.broker_id.unwrap(), 3);
        assert_eq!(result.connection_id.unwrap(), 3);

        let result = session_storage
            .list_session(Some(client_id.clone()))
            .await
            .unwrap();

        assert!(!result.is_empty());

        session_storage
            .delete_session(client_id.clone())
            .await
            .unwrap();

        let result = session_storage
            .get_session(client_id.clone())
            .await
            .unwrap();
        assert!(result.is_none());

        let result = session_storage
            .list_session(Some(client_id.clone()))
            .await
            .unwrap();
        assert!(result.is_empty());
    }
}
