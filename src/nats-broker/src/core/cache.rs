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

use crate::core::connection::NatsConnection;
use broker_core::cache::NodeCacheManager;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::nats::subject::NatsSubject;
use std::sync::Arc;

pub struct NatsCacheManager {
    // broker cache
    pub node_cache: Arc<NodeCacheManager>,

    // client pool
    pub client_pool: Arc<ClientPool>,

    // (connect_id, NatsConnection)
    pub connection_info: DashMap<u64, NatsConnection>,

    // ("{tenant}/{name}", NatsSubject)
    pub subject_info: DashMap<String, NatsSubject>,
}

impl NatsCacheManager {
    pub fn new(client_pool: Arc<ClientPool>, node_cache: Arc<NodeCacheManager>) -> Self {
        NatsCacheManager {
            node_cache,
            client_pool,
            connection_info: DashMap::with_capacity(1024),
            subject_info: DashMap::with_capacity(256),
        }
    }

    // connection

    pub fn add_connection(&self, connection: NatsConnection) {
        self.connection_info
            .insert(connection.connect_id, connection);
    }

    pub fn remove_connection(&self, connect_id: u64) {
        self.connection_info.remove(&connect_id);
    }

    pub fn get_connection(&self, connect_id: u64) -> Option<NatsConnection> {
        self.connection_info
            .get(&connect_id)
            .map(|e| e.value().clone())
    }

    pub fn get_connection_count(&self) -> usize {
        self.connection_info.len()
    }

    pub fn login_success(&self, connect_id: u64, user_name: String) {
        if let Some(mut conn) = self.connection_info.get_mut(&connect_id) {
            conn.login_success(user_name);
        }
    }

    pub fn is_login(&self, connect_id: u64) -> bool {
        self.connection_info
            .get(&connect_id)
            .map(|e| e.is_login)
            .unwrap_or(false)
    }

    // subject

    pub fn add_subject(&self, subject: NatsSubject) {
        let key = format!("{}/{}", subject.tenant, subject.name);
        self.subject_info.insert(key, subject);
    }

    pub fn remove_subject(&self, tenant: &str, name: &str) {
        let key = format!("{}/{}", tenant, name);
        self.subject_info.remove(&key);
    }

    pub fn get_subject(&self, tenant: &str, name: &str) -> Option<NatsSubject> {
        let key = format!("{}/{}", tenant, name);
        self.subject_info.get(&key).map(|e| e.value().clone())
    }

    pub fn list_subjects_by_tenant(&self, tenant: &str) -> Vec<NatsSubject> {
        let prefix = format!("{}/", tenant);
        self.subject_info
            .iter()
            .filter(|e| e.key().starts_with(&prefix))
            .map(|e| e.value().clone())
            .collect()
    }
}
