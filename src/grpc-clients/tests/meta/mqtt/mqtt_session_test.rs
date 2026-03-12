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
    use common_base::uuid::unique_id;
    use grpc_clients::meta::mqtt::call::{
        placement_create_session, placement_delete_session, placement_list_session,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::session::MqttSession;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use protocol::meta::meta_service_mqtt::{
        CreateSessionRaw, CreateSessionRequest, DeleteSessionRequest, ListSessionRequest,
    };
    use std::sync::Arc;

    fn build_session(tenant: &str, client_id: &str) -> MqttSession {
        let mut session = MqttSession::new(
            tenant.to_string(),
            client_id.to_string(),
            10000,
            false,
            None,
            true,
        );
        session.update_broker_id(Some(1));
        session.update_connection_id(Some(1));
        session
    }

    async fn create_sessions(
        client_pool: &Arc<ClientPool>,
        addrs: &[String],
        sessions: Vec<MqttSession>,
    ) {
        let raws = sessions
            .iter()
            .map(|s| CreateSessionRaw {
                client_id: s.client_id.clone(),
                session: s.encode().unwrap(),
            })
            .collect();
        placement_create_session(client_pool, addrs, CreateSessionRequest { sessions: raws })
            .await
            .unwrap();
    }

    async fn list_sessions(
        client_pool: &Arc<ClientPool>,
        addrs: &[String],
        tenant: &str,
        client_id: &str,
    ) -> Vec<MqttSession> {
        let request = ListSessionRequest {
            tenant: tenant.to_string(),
            client_id: client_id.to_string(),
        };
        let mut stream = placement_list_session(client_pool, addrs, request)
            .await
            .unwrap();
        let mut results = Vec::new();
        while let Some(reply) = stream.message().await.unwrap() {
            results.push(MqttSession::decode(&reply.session).unwrap());
        }
        results
    }

    #[tokio::test]
    async fn mqtt_session_crud_test() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let client_id = unique_id();

        let session = build_session(DEFAULT_TENANT, &client_id);
        create_sessions(&client_pool, &addrs, vec![session.clone()]).await;

        // verify session exists
        let results = list_sessions(&client_pool, &addrs, DEFAULT_TENANT, &client_id).await;
        assert!(results.iter().any(|s| s == &session));

        // delete and verify gone
        placement_delete_session(
            &client_pool,
            &addrs,
            DeleteSessionRequest {
                tenant: DEFAULT_TENANT.to_string(),
                client_id: client_id.clone(),
            },
        )
        .await
        .unwrap();

        let results = list_sessions(&client_pool, &addrs, DEFAULT_TENANT, &client_id).await;
        assert!(!results.iter().any(|s| s.client_id == client_id));
    }

    #[tokio::test]
    async fn mqtt_session_tenant_isolation_test() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];

        let tenant_a = format!("tenant-a-{}", unique_id());
        let tenant_b = format!("tenant-b-{}", unique_id());
        let client_id_1 = unique_id();
        let client_id_2 = unique_id();

        let session_a1 = build_session(&tenant_a, &client_id_1);
        let session_a2 = build_session(&tenant_a, &client_id_2);
        let session_b1 = build_session(&tenant_b, &client_id_1);

        create_sessions(
            &client_pool,
            &addrs,
            vec![session_a1.clone(), session_a2.clone(), session_b1.clone()],
        )
        .await;

        // list by tenant_a: should return only tenant_a sessions
        let results_a = list_sessions(&client_pool, &addrs, &tenant_a, "").await;
        assert_eq!(results_a.len(), 2);
        assert!(results_a.iter().all(|s| s.tenant == tenant_a));

        // list by tenant_b: should return only tenant_b sessions
        let results_b = list_sessions(&client_pool, &addrs, &tenant_b, "").await;
        assert_eq!(results_b.len(), 1);
        assert_eq!(results_b[0].tenant, tenant_b);

        // get specific session by tenant + client_id
        let results = list_sessions(&client_pool, &addrs, &tenant_a, &client_id_1).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], session_a1);

        // same client_id under tenant_b returns different session
        let results = list_sessions(&client_pool, &addrs, &tenant_b, &client_id_1).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], session_b1);
    }
}
