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

use crate::{
    mqtt::{admin::MqttBrokerAdminServiceManager, placement::MqttBrokerPlacementServiceManager},
    placement::{
        journal::JournalServiceManager, kv::KvServiceManager, mqtt::MQTTServiceManager,
        placement::PlacementServiceManager,
    },
};
use common_base::error::common::CommonError;
use dashmap::DashMap;
use log::debug;
use mobc::Pool;
use protocol::{
    broker_server::generate::{
        admin::mqtt_broker_admin_service_client::MqttBrokerAdminServiceClient,
        placement::mqtt_broker_placement_service_client::MqttBrokerPlacementServiceClient,
    },
    placement_center::generate::{
        journal::engine_service_client::EngineServiceClient,
        kv::kv_service_client::KvServiceClient, mqtt::mqtt_service_client::MqttServiceClient,
        placement::placement_center_service_client::PlacementCenterServiceClient,
    },
};
use tonic::transport::Channel;

#[derive(Clone)]
pub struct ClientPool {
    max_open_connection: u64,
    // placement center
    placement_center_inner_pools: DashMap<String, Pool<PlacementServiceManager>>,
    placement_center_journal_service_pools: DashMap<String, Pool<JournalServiceManager>>,
    placement_center_kv_service_pools: DashMap<String, Pool<KvServiceManager>>,
    placement_center_mqtt_service_pools: DashMap<String, Pool<MQTTServiceManager>>,

    // mqtt broker
    mqtt_broker_placement_service_pools: DashMap<String, Pool<MqttBrokerPlacementServiceManager>>,
    mqtt_broker_admin_service_pools: DashMap<String, Pool<MqttBrokerAdminServiceManager>>,
}

impl ClientPool {
    pub fn new(max_open_connection: u64) -> Self {
        Self {
            max_open_connection,
            placement_center_inner_pools: DashMap::with_capacity(128),
            placement_center_journal_service_pools: DashMap::with_capacity(128),
            placement_center_kv_service_pools: DashMap::with_capacity(128),
            placement_center_mqtt_service_pools: DashMap::with_capacity(128),
            mqtt_broker_placement_service_pools: DashMap::with_capacity(128),
            mqtt_broker_admin_service_pools: DashMap::with_capacity(128),
        }
    }

    pub async fn placement_center_inner_services_client(
        &self,
        addr: String,
    ) -> Result<PlacementCenterServiceClient<Channel>, CommonError> {
        let module = "PlacementService".to_string();
        let key = format!("{}_{}_{}", "PlacementServer", module, addr);
        if !self.placement_center_inner_pools.contains_key(&key) {
            let manager = PlacementServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.max_open_connection).build(manager);
            self.placement_center_inner_pools.insert(key.clone(), pool);
        }
        if let Some(poll) = self.placement_center_inner_pools.get(&key) {
            debug!("{:?}", poll.state().await);
            match poll.get().await {
                Ok(conn) => return Ok(conn.clone()),
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(module, e.to_string()));
                }
            };
        }
        return Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }

    pub async fn placement_center_journal_services_client(
        &self,
        addr: String,
    ) -> Result<EngineServiceClient<Channel>, CommonError> {
        let module = "JournalService".to_string();
        let key = format!("{}_{}_{}", "JournalServer", module, addr);
        if !self.placement_center_journal_service_pools.contains_key(&key) {
            let manager = JournalServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.max_open_connection).build(manager);
            self.placement_center_journal_service_pools.insert(key.clone(), pool);
        }
        if let Some(poll) = self.placement_center_journal_service_pools.get(&key) {
            
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.clone());
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(module, e.to_string()));
                }
            };
        }
        return Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }

    pub async fn placement_center_kv_services_client(
        &self,
        addr: String,
    ) -> Result<KvServiceClient<Channel>, CommonError> {
        let module = "KvServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);

        if !self.placement_center_kv_service_pools.contains_key(&key) {
            let manager = KvServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.max_open_connection).build(manager);
            self.placement_center_kv_service_pools.insert(key.clone(), pool);
        }

        if let Some(poll) = self.placement_center_kv_service_pools.get(&key) {
            debug!("{:?}", poll.state().await);
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.clone());
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(module, e.to_string()));
                }
            };
        }

        return Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }

    pub async fn placement_center_mqtt_services_client(
        &self,
        addr: String,
    ) -> Result<MqttServiceClient<Channel>, CommonError> {
        let module = "MqttServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);

        if !self.placement_center_kv_service_pools.contains_key(&key) {
            let manager = MQTTServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.max_open_connection).build(manager);
            self.placement_center_mqtt_service_pools.insert(key.clone(), pool);
        }

        if let Some(poll) = self.placement_center_mqtt_service_pools.get(&key) {
            debug!("{:?}", poll.state().await);
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.clone());
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(module, e.to_string()));
                }
            };
        }
        return Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }

    pub async fn mqtt_broker_mqtt_services_client(
        &self,
        addr: String,
    ) -> Result<MqttBrokerPlacementServiceClient<Channel>, CommonError> {
        let module = "BrokerPlacementServices".to_string();
        let key = format!("{}_{}_{}", "MQTTBroker", module, addr);

        if !self.mqtt_broker_placement_service_pools.contains_key(&key) {
            let manager = MqttBrokerPlacementServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.max_open_connection).build(manager);
            self.mqtt_broker_placement_service_pools.insert(key.clone(), pool);
        }

        if let Some(poll) = self.mqtt_broker_placement_service_pools.get(&key) {
            debug!("{:?}", poll.state().await);
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.clone());
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(module, e.to_string()));
                }
            };
        }
        return Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }

    pub async fn mqtt_broker_admin_services_client(
        &self,
        addr: String,
    ) -> Result<MqttBrokerAdminServiceClient<Channel>, CommonError> {
        let module = "BrokerAdminServices".to_string();
        let key = format!("{}_{}_{}", "MQTTBroker", module, addr);

        if !self.mqtt_broker_placement_service_pools.contains_key(&key) {
            let manager = MqttBrokerAdminServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.max_open_connection).build(manager);
            self.mqtt_broker_admin_service_pools.insert(key.clone(), pool);
        }

        if let Some(poll) = self.mqtt_broker_admin_service_pools.get(&key) {
            debug!("{:?}", poll.state().await);
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.clone());
                }
                Err(e) => {
                    return Err(CommonError::NoAvailableGrpcConnection(module, e.to_string()));
                }
            };
        }
        return Err(CommonError::NoAvailableGrpcConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn get_placement_center_inner_services_client_test() {}
}
