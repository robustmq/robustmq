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

use std::time::Duration;

use crate::journal::admin::JournalAdminServiceManager;
use crate::journal::inner::JournalInnerServiceManager;
use crate::meta::inner::PlacementServiceManager;
use crate::meta::journal::JournalServiceManager;
use crate::meta::kv::KvServiceManager;
use crate::meta::mqtt::MqttServiceManager;
use crate::meta::openraft::OpenRaftServiceManager;
use crate::mqtt::inner::MqttBrokerPlacementServiceManager;
use common_base::error::common::CommonError;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use mobc::{Connection, Pool};
use tracing::info;

#[derive(Clone)]
pub struct ClientPool {
    max_open_connection: u64,
    // modules: meta service
    meta_service_inner_pools: DashMap<String, Pool<PlacementServiceManager>>,
    meta_service_journal_service_pools: DashMap<String, Pool<JournalServiceManager>>,
    meta_service_kv_service_pools: DashMap<String, Pool<KvServiceManager>>,
    meta_service_mqtt_service_pools: DashMap<String, Pool<MqttServiceManager>>,
    meta_service_openraft_service_pools: DashMap<String, Pool<OpenRaftServiceManager>>,
    // modules: meta service service: leader cache
    meta_service_leader_addr_caches: DashMap<String, String>,

    // modules: mqtt broker
    mqtt_broker_placement_service_pools: DashMap<String, Pool<MqttBrokerPlacementServiceManager>>,

    // modules: journal engine
    journal_admin_service_pools: DashMap<String, Pool<JournalAdminServiceManager>>,
    journal_inner_service_pools: DashMap<String, Pool<JournalInnerServiceManager>>,
}

impl ClientPool {
    pub fn new(max_open_connection: u64) -> Self {
        Self {
            max_open_connection,
            // modules: meta_service
            meta_service_inner_pools: DashMap::with_capacity(2),
            meta_service_journal_service_pools: DashMap::with_capacity(2),
            meta_service_kv_service_pools: DashMap::with_capacity(2),
            meta_service_mqtt_service_pools: DashMap::with_capacity(2),
            meta_service_openraft_service_pools: DashMap::with_capacity(2),
            meta_service_leader_addr_caches: DashMap::with_capacity(2),
            // modules: mqtt_broker
            mqtt_broker_placement_service_pools: DashMap::with_capacity(2),
            // modules: journal_engine
            journal_admin_service_pools: DashMap::with_capacity(2),
            journal_inner_service_pools: DashMap::with_capacity(2),
        }
    }

    // ----------modules: meta service -------------
    pub async fn meta_service_inner_services_client(
        &self,
        addr: &str,
    ) -> Result<Connection<PlacementServiceManager>, CommonError> {
        if !self.meta_service_inner_pools.contains_key(addr) {
            let manager = PlacementServiceManager::new(addr.to_owned());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.meta_service_inner_pools.insert(addr.to_owned(), pool);
        }

        if let Some(pool) = self.meta_service_inner_pools.get(addr) {
            match pool.get_timeout(Duration::from_secs(3)).await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "PlacementService".to_string(),
                        format!(
                            "get meta service inner service client failed, err: {}, state: {:?}",
                            e,
                            pool.state().await
                        ),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "PlacementService".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn meta_service_journal_services_client(
        &self,
        addr: &str,
    ) -> Result<Connection<JournalServiceManager>, CommonError> {
        if !self.meta_service_journal_service_pools.contains_key(addr) {
            let manager = JournalServiceManager::new(addr.to_owned());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.meta_service_journal_service_pools
                .insert(addr.to_owned(), pool);
        }
        if let Some(pool) = self.meta_service_journal_service_pools.get(addr) {
            match pool.get_timeout(Duration::from_secs(3)).await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "JournalService".to_string(),
                        format!(
                            "get meta service journal service client failed, err: {}, state: {:?}",
                            e,
                            pool.state().await
                        ),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "JournalService".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn meta_service_kv_services_client(
        &self,
        addr: &str,
    ) -> Result<Connection<KvServiceManager>, CommonError> {
        if !self.meta_service_kv_service_pools.contains_key(addr) {
            let manager = KvServiceManager::new(addr.to_owned());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.meta_service_kv_service_pools
                .insert(addr.to_owned(), pool);
        }

        if let Some(pool) = self.meta_service_kv_service_pools.get(addr) {
            match pool.get_timeout(Duration::from_secs(3)).await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "KvServices".to_string(),
                        format!(
                            "get meta service kv service client failed, err: {}, state: {:?}",
                            e,
                            pool.state().await
                        ),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            "KvServices".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn meta_service_mqtt_services_client(
        &self,
        addr: &str,
    ) -> Result<Connection<MqttServiceManager>, CommonError> {
        if !self.meta_service_mqtt_service_pools.contains_key(addr) {
            let manager = MqttServiceManager::new(addr.to_owned());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.meta_service_mqtt_service_pools
                .insert(addr.to_owned(), pool);
        }
        if let Some(pool) = self.meta_service_mqtt_service_pools.get(addr) {
            match pool.get_timeout(Duration::from_secs(3)).await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "MqttServices".to_string(),
                        format!(
                            "get meta service mqtt service client failed, err: {}, state: {:?}",
                            e,
                            pool.state().await
                        ),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "MqttServices".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn meta_service_openraft_services_client(
        &self,
        addr: &str,
    ) -> Result<Connection<OpenRaftServiceManager>, CommonError> {
        if !self.meta_service_openraft_service_pools.contains_key(addr) {
            let manager = OpenRaftServiceManager::new(addr.to_owned());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.meta_service_openraft_service_pools
                .insert(addr.to_owned(), pool);
        }

        if let Some(pool) = self.meta_service_openraft_service_pools.get(addr) {
            match pool.get_timeout(Duration::from_secs(3)).await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "OpenRaftServices".to_string(),
                        format!(
                            "get meta service openraft service client failed, err: {}, state: {:?}",
                            e,
                            pool.state().await
                        ),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            "OpenRaftServices".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    // ----------modules: mqtt broker -------------
    pub async fn mqtt_broker_mqtt_services_client(
        &self,
        addr: &str,
    ) -> Result<Connection<MqttBrokerPlacementServiceManager>, CommonError> {
        if !self.mqtt_broker_placement_service_pools.contains_key(addr) {
            let manager = MqttBrokerPlacementServiceManager::new(addr.to_owned());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.mqtt_broker_placement_service_pools
                .insert(addr.to_owned(), pool);
        }

        if let Some(pool) = self.mqtt_broker_placement_service_pools.get(addr) {
            match pool.get_timeout(Duration::from_secs(3)).await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "MQTTBrokerPlacementServices".to_string(),
                        format!(
                            "get mqtt broker mqtt service client failed, err: {}, state: {:?}",
                            e,
                            pool.state().await
                        ),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "MQTTBrokerPlacementServices".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    // ----------modules: journal engine -------------
    pub async fn journal_inner_services_client(
        &self,
        addr: &str,
    ) -> Result<Connection<JournalInnerServiceManager>, CommonError> {
        if !self.journal_inner_service_pools.contains_key(addr) {
            let manager = JournalInnerServiceManager::new(addr.to_owned());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.journal_inner_service_pools
                .insert(addr.to_owned(), pool);
        }

        if let Some(pool) = self.journal_inner_service_pools.get(addr) {
            match pool.get_timeout(Duration::from_secs(3)).await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "JournalEngine".to_string(),
                        format!(
                            "get journal engine inner service client failed, err: {}, state: {:?}",
                            e,
                            pool.state().await
                        ),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            "JournalEngine".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn journal_admin_services_client(
        &self,
        addr: &str,
    ) -> Result<Connection<JournalAdminServiceManager>, CommonError> {
        if !self.journal_admin_service_pools.contains_key(addr) {
            let manager = JournalAdminServiceManager::new(addr.to_owned());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.journal_admin_service_pools
                .insert(addr.to_owned().to_owned(), pool);
        }

        if let Some(pool) = self.journal_admin_service_pools.get(addr) {
            match pool.get_timeout(Duration::from_secs(3)).await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "JournalEngineAdmin".to_string(),
                        format!(
                            "get journal engine admin service client failed, err: {}, state: {:?}",
                            e,
                            pool.state().await
                        ),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            "JournalEngineAdmin".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    // other
    pub fn get_leader_addr(&self, addr: &str) -> Option<Ref<'_, String, String>> {
        self.meta_service_leader_addr_caches.get(addr)
    }

    pub fn set_leader_addr(&self, addr: String, leader_addr: String) {
        info!(
            "Update the Leader information in the client cache with the new Leader address :{}",
            leader_addr
        );
        self.meta_service_leader_addr_caches
            .insert(addr.to_owned(), leader_addr);
    }
}
