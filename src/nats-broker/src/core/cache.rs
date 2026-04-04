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
use metadata_struct::nats::subscribe::NatsSubscribe;
use std::sync::Arc;

pub struct NatsCacheManager {
    // broker cache
    pub node_cache: Arc<NodeCacheManager>,

    // client pool
    pub client_pool: Arc<ClientPool>,

    // (connect_id, NatsConnection)
    pub connection_info: DashMap<u64, NatsConnection>,

    // ("{client_id}/{sid}", NatsSubscribe)
    pub subscribe_info: DashMap<String, NatsSubscribe>,
}

impl NatsCacheManager {
    pub fn new(client_pool: Arc<ClientPool>, node_cache: Arc<NodeCacheManager>) -> Self {
        NatsCacheManager {
            node_cache,
            client_pool,
            connection_info: DashMap::with_capacity(1024),
            subscribe_info: DashMap::with_capacity(256),
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

    // subscribe
    pub fn add_subscribe(&self, subscribe: NatsSubscribe) {
        let key = format!("{}/{}", subscribe.connect_id, subscribe.sid);
        self.subscribe_info.insert(key, subscribe);
    }

    pub fn remove_subscribe(&self, connect_id: u64, sid: &str) {
        let key = format!("{}/{}", connect_id, sid);
        self.subscribe_info.remove(&key);
    }

    pub fn get_subscribe(&self, connect_id: u64, sid: &str) -> Option<NatsSubscribe> {
        let key = format!("{}/{}", connect_id, sid);
        self.subscribe_info.get(&key).map(|e| e.value().clone())
    }

    pub fn list_subscribes_by_connection(&self, connect_id: u64) -> Vec<NatsSubscribe> {
        let prefix = format!("{}/", connect_id);
        self.subscribe_info
            .iter()
            .filter(|e| e.key().starts_with(&prefix))
            .map(|e| e.value().clone())
            .collect()
    }
}
