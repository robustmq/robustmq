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

use std::net::SocketAddr;

use common_base::error::common::CommonError;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use log::info;
use mobc::{Connection, Pool};

use crate::journal::admin::JournalAdminServiceManager;
use crate::journal::inner::JournalInnerServiceManager;
use crate::mqtt::admin::MqttBrokerAdminServiceManager;
use crate::mqtt::inner::MqttBrokerPlacementServiceManager;
use crate::placement::inner::PlacementServiceManager;
use crate::placement::journal::JournalServiceManager;
use crate::placement::kv::KvServiceManager;
use crate::placement::mqtt::MqttServiceManager;
use crate::placement::openraft::OpenRaftServiceManager;

#[derive(Clone)]
pub struct ClientPool {
    max_open_connection: u64,
    // modules: placement center
    placement_center_inner_pools: DashMap<SocketAddr, Pool<PlacementServiceManager>>,
    placement_center_journal_service_pools: DashMap<SocketAddr, Pool<JournalServiceManager>>,
    placement_center_kv_service_pools: DashMap<SocketAddr, Pool<KvServiceManager>>,
    placement_center_mqtt_service_pools: DashMap<SocketAddr, Pool<MqttServiceManager>>,
    placement_center_openraft_service_pools: DashMap<SocketAddr, Pool<OpenRaftServiceManager>>,
    // modules: placement center service: leader cache
    placement_center_leader_addr_caches: DashMap<SocketAddr, SocketAddr>,

    // modules: mqtt broker
    mqtt_broker_placement_service_pools: DashMap<SocketAddr, Pool<MqttBrokerPlacementServiceManager>>,
    mqtt_broker_admin_service_pools: DashMap<SocketAddr, Pool<MqttBrokerAdminServiceManager>>,

    // modules: journal engine
    journal_admin_service_pools: DashMap<SocketAddr, Pool<JournalAdminServiceManager>>,
    journal_inner_service_pools: DashMap<SocketAddr, Pool<JournalInnerServiceManager>>,
}

impl ClientPool {
    pub fn new(max_open_connection: u64) -> Self {
        Self {
            max_open_connection,
            // modules: placement_center
            placement_center_inner_pools: DashMap::with_capacity(2),
            placement_center_journal_service_pools: DashMap::with_capacity(2),
            placement_center_kv_service_pools: DashMap::with_capacity(2),
            placement_center_mqtt_service_pools: DashMap::with_capacity(2),
            placement_center_openraft_service_pools: DashMap::with_capacity(2),
            placement_center_leader_addr_caches: DashMap::with_capacity(2),
            // modules: mqtt_broker
            mqtt_broker_placement_service_pools: DashMap::with_capacity(2),
            mqtt_broker_admin_service_pools: DashMap::with_capacity(2),
            // modules: journal_engine
            journal_admin_service_pools: DashMap::with_capacity(2),
            journal_inner_service_pools: DashMap::with_capacity(2),
        }
    }

    // ----------modules: placement center -------------
    pub async fn placement_center_inner_services_client(
        &self,
        addr: SocketAddr,
    ) -> Result<Connection<PlacementServiceManager>, CommonError> {
        if !self.placement_center_inner_pools.contains_key(&addr) {
            let manager = PlacementServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_inner_pools.insert(addr, pool);
        }
        if let Some(pool) = self.placement_center_inner_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "PlacementService".to_string(),
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "PlacementService".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn placement_center_journal_services_client(
        &self,
        addr: SocketAddr
    ) -> Result<Connection<JournalServiceManager>, CommonError> {
        if !self
            .placement_center_journal_service_pools
            .contains_key(&addr)
        {
            let manager = JournalServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_journal_service_pools
                .insert(addr, pool);
        }
        if let Some(pool) = self.placement_center_journal_service_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "JournalService".to_string(),
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "JournalService".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn placement_center_kv_services_client(
        &self,
        addr: SocketAddr
    ) -> Result<Connection<KvServiceManager>, CommonError> {
        if !self.placement_center_kv_service_pools.contains_key(&addr) {
            let manager = KvServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_kv_service_pools
                .insert(addr, pool);
        }

        if let Some(pool) = self.placement_center_kv_service_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "KvServices".to_string(),
                        e.to_string(),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            "KvServices".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn placement_center_mqtt_services_client(
        &self,
        addr: SocketAddr
    ) -> Result<Connection<MqttServiceManager>, CommonError> {
        if !self.placement_center_mqtt_service_pools.contains_key(&addr) {
            let manager = MqttServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_mqtt_service_pools
                .insert(addr, pool);
        }
        if let Some(pool) = self.placement_center_mqtt_service_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "MqttServices".to_string(),
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "MqttServices".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn placement_center_openraft_services_client(
        &self,
        addr: SocketAddr
    ) -> Result<Connection<OpenRaftServiceManager>, CommonError> {
        if !self
            .placement_center_openraft_service_pools
            .contains_key(&addr)
        {
            let manager = OpenRaftServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_openraft_service_pools
                .insert(addr, pool);
        }

        if let Some(pool) = self.placement_center_openraft_service_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "OpenRaftServices".to_string(),
                        e.to_string(),
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
        addr: SocketAddr
    ) -> Result<Connection<MqttBrokerPlacementServiceManager>, CommonError> {
        if !self.mqtt_broker_placement_service_pools.contains_key(&addr) {
            let manager = MqttBrokerPlacementServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.mqtt_broker_placement_service_pools
                .insert(addr, pool);
        }

        if let Some(pool) = self.mqtt_broker_placement_service_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "BrokerPlacementServices".to_string(),
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "BrokerPlacementServices".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    pub async fn mqtt_broker_admin_services_client(
        &self,
        addr: SocketAddr
    ) -> Result<Connection<MqttBrokerAdminServiceManager>, CommonError> {
        if !self.mqtt_broker_admin_service_pools.contains_key(&addr) {
            let manager = MqttBrokerAdminServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.mqtt_broker_admin_service_pools
                .insert(addr, pool);
        }

        if let Some(pool) = self.mqtt_broker_admin_service_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "BrokerAdminServices".to_string(),
                        e.to_string(),
                    ));
                }
            };
        }
        Err(CommonError::NoAvailableGrpcConnection(
            "BrokerAdminServices".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    // ----------modules: journal engine -------------
    pub async fn journal_inner_services_client(
        &self,
        addr: SocketAddr
    ) -> Result<Connection<JournalInnerServiceManager>, CommonError> {
        if !self.journal_inner_service_pools.contains_key(&addr) {
            let manager = JournalInnerServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.journal_inner_service_pools.insert(addr, pool);
        }

        if let Some(pool) = self.journal_inner_service_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "JournalEngine".to_string(),
                        e.to_string(),
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
        addr: SocketAddr
    ) -> Result<Connection<JournalAdminServiceManager>, CommonError> {
        if !self.journal_admin_service_pools.contains_key(&addr) {
            let manager = JournalAdminServiceManager::new(addr);
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.journal_admin_service_pools.insert(addr, pool);
        }

        if let Some(pool) = self.journal_admin_service_pools.get(&addr) {
            match pool.get().await {
                Ok(conn) => {
                    return Ok(conn);
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(
                        "JournalEngine".to_string(),
                        e.to_string(),
                    ));
                }
            };
        }

        Err(CommonError::NoAvailableGrpcConnection(
            "JournalEngine".to_string(),
            "connection pool is not initialized".to_string(),
        ))
    }

    // other
    pub fn get_leader_addr(&self, addr: SocketAddr) -> Option<Ref<'_, SocketAddr, SocketAddr>> {
        self.placement_center_leader_addr_caches.get(&addr)
    }

    pub fn set_leader_addr(&self, addr: SocketAddr, leader_addr: SocketAddr) {
        info!(
            "Update the Leader information in the client cache with the new Leader address :{}",
            leader_addr
        );
        self.placement_center_leader_addr_caches
            .insert(addr, leader_addr);
    }
}
