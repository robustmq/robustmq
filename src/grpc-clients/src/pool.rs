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

use common_base::error::common::CommonError;
use dashmap::DashMap;
use log::info;
use mobc::{Connection, Pool};

use crate::journal::admin::JournalAdminServiceManager;
use crate::journal::inner::JournalInnerServiceManager;
use crate::mqtt::admin::MqttBrokerAdminServiceManager;
use crate::mqtt::placement::MqttBrokerPlacementServiceManager;
use crate::placement::journal::JournalServiceManager;
use crate::placement::kv::KvServiceManager;
use crate::placement::mqtt::MqttServiceManager;
use crate::placement::openraft::OpenRaftServiceManager;
use crate::placement::placement::PlacementServiceManager;

#[derive(Clone)]
pub struct ClientPool {
    max_open_connection: u64,
    // placement center
    placement_center_inner_pools: DashMap<String, Pool<PlacementServiceManager>>,
    placement_center_journal_service_pools: DashMap<String, Pool<JournalServiceManager>>,
    placement_center_kv_service_pools: DashMap<String, Pool<KvServiceManager>>,
    placement_center_mqtt_service_pools: DashMap<String, Pool<MqttServiceManager>>,
    placement_center_openraft_service_pools: DashMap<String, Pool<OpenRaftServiceManager>>,

    // mqtt broker
    mqtt_broker_placement_service_pools: DashMap<String, Pool<MqttBrokerPlacementServiceManager>>,
    mqtt_broker_admin_service_pools: DashMap<String, Pool<MqttBrokerAdminServiceManager>>,

    // journal engine
    journal_admin_service_pools: DashMap<String, Pool<JournalAdminServiceManager>>,
    journal_inner_service_pools: DashMap<String, Pool<JournalInnerServiceManager>>,

    // leader cache
    placement_center_leader_addr_caches: DashMap<String, String>,
}

impl ClientPool {
    pub fn new(max_open_connection: u64) -> Self {
        Self {
            max_open_connection,
            placement_center_inner_pools: DashMap::with_capacity(2),
            placement_center_journal_service_pools: DashMap::with_capacity(2),
            placement_center_kv_service_pools: DashMap::with_capacity(2),
            placement_center_mqtt_service_pools: DashMap::with_capacity(2),
            mqtt_broker_placement_service_pools: DashMap::with_capacity(2),
            mqtt_broker_admin_service_pools: DashMap::with_capacity(2),
            placement_center_openraft_service_pools: DashMap::with_capacity(2),
            placement_center_leader_addr_caches: DashMap::with_capacity(2),
            journal_admin_service_pools: DashMap::with_capacity(2),
            journal_inner_service_pools: DashMap::with_capacity(2),
        }
    }

    pub async fn placement_center_inner_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<PlacementServiceManager>, CommonError> {
        let module = "PlacementService".to_string();
        let key = format!("{}_{}_{}", "PlacementServer", module, addr);
        if !self.placement_center_inner_pools.contains_key(&key) {
            let manager = PlacementServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_inner_pools.insert(key.clone(), pool);
        }
        if let Some(pool) = self.placement_center_inner_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn placement_center_journal_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<JournalServiceManager>, CommonError> {
        let module = "JournalService".to_string();
        let key = format!("{}_{}_{}", "JournalServer", module, addr);
        if !self
            .placement_center_journal_service_pools
            .contains_key(&key)
        {
            let manager = JournalServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_journal_service_pools
                .insert(key.clone(), pool);
        }
        if let Some(pool) = self.placement_center_journal_service_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn placement_center_kv_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<KvServiceManager>, CommonError> {
        let module = "KvServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);

        if !self.placement_center_kv_service_pools.contains_key(&key) {
            let manager = KvServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_kv_service_pools
                .insert(key.clone(), pool);
        }

        if let Some(pool) = self.placement_center_kv_service_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn placement_center_mqtt_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<MqttServiceManager>, CommonError> {
        let module = "MqttServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);

        if !self.placement_center_mqtt_service_pools.contains_key(&key) {
            let manager = MqttServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_mqtt_service_pools
                .insert(key.clone(), pool);
        }
        if let Some(pool) = self.placement_center_mqtt_service_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn mqtt_broker_mqtt_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<MqttBrokerPlacementServiceManager>, CommonError> {
        let module = "BrokerPlacementServices".to_string();
        let key = format!("{}_{}_{}", "MQTTBroker", module, addr);

        if !self.mqtt_broker_placement_service_pools.contains_key(&key) {
            let manager = MqttBrokerPlacementServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.mqtt_broker_placement_service_pools
                .insert(key.clone(), pool);
        }

        if let Some(pool) = self.mqtt_broker_placement_service_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn mqtt_broker_admin_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<MqttBrokerAdminServiceManager>, CommonError> {
        let module = "BrokerAdminServices".to_string();
        let key = format!("{}_{}_{}", "MQTTBroker", module, addr);

        if !self.mqtt_broker_admin_service_pools.contains_key(&key) {
            let manager = MqttBrokerAdminServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.mqtt_broker_admin_service_pools
                .insert(key.clone(), pool);
        }

        if let Some(pool) = self.mqtt_broker_admin_service_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn placement_center_openraft_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<OpenRaftServiceManager>, CommonError> {
        let module = "OpenRaftServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);
        if !self
            .placement_center_openraft_service_pools
            .contains_key(&key)
        {
            let manager = OpenRaftServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_openraft_service_pools
                .insert(key.clone(), pool);
        }

        if let Some(pool) = self.placement_center_openraft_service_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn journal_inner_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<JournalInnerServiceManager>, CommonError> {
        let module = "inner".to_string();
        let key = format!("{}_{}_{}", "JournalEngine", module, addr);
        if !self.journal_inner_service_pools.contains_key(&key) {
            let manager = JournalInnerServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.journal_inner_service_pools.insert(key.clone(), pool);
        }

        if let Some(pool) = self.journal_inner_service_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn journal_admin_services_client(
        &self,
        addr: String,
    ) -> Result<Connection<JournalAdminServiceManager>, CommonError> {
        let module = "admin".to_string();
        let key = format!("{}_{}_{}", "JournalEngine", module, addr);
        if !self.journal_admin_service_pools.contains_key(&key) {
            let manager = JournalAdminServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.journal_admin_service_pools.insert(key.clone(), pool);
        }

        if let Some(pool) = self.journal_admin_service_pools.get(&key) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        module,
                        e.to_string(),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ))
    }

    pub fn get_leader_addr(&self, addr: &str) -> Option<String> {
        if let Some(leader_addr) = self.placement_center_leader_addr_caches.get(addr) {
            return Some(leader_addr.clone());
        }
        None
    }

    pub fn set_leader_addr(&self, addr: String, leader_addr: String) {
        info!(
            "Update the Leader information in the client cache with the new Leader address :{}",
            leader_addr
        );
        self.placement_center_leader_addr_caches
            .insert(addr, leader_addr);
    }
}
