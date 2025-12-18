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

use crate::broker::common::BrokerCommonServiceManager;
use crate::broker::storage::BrokerStorageServiceManager;
use crate::meta::journal::JournalServiceManager;
use crate::meta::mqtt::MqttServiceManager;
use crate::{broker::mqtt::BrokerMqttServiceManager, meta::common::PlacementServiceManager};
use common_base::error::common::CommonError;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use mobc::{Connection, Pool};
use std::time::Duration;
use tracing::{debug, info, warn};

// Increased default timeout to handle network latency better
const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 10;

macro_rules! define_client_method {
    (
        $method_name:ident,
        $pool_field:ident,
        $manager:ty,
        $service_name:expr
    ) => {
        pub async fn $method_name(&self, addr: &str) -> Result<Connection<$manager>, CommonError> {
            // Initialize pool if not exists
            if !self.$pool_field.contains_key(addr) {
                debug!("Creating new connection pool for {} at {}", $service_name, addr);
                let manager = <$manager>::new(addr.to_owned());
                let pool = Pool::builder()
                    .max_open(self.max_open_connection)
                    .build(manager);
                self.$pool_field.insert(addr.to_owned(), pool);
                info!("Connection pool for {} at {} initialized (max_open: {}, timeout: {:?})",
                    $service_name, addr, self.max_open_connection, self.connection_timeout);
            }

            if let Some(pool) = self.$pool_field.get(addr) {
                let pool_state_before = pool.state().await;
                debug!("Attempting to get connection from {} pool at {} (state: {:?})",
                    $service_name, addr, pool_state_before);

                match pool.get_timeout(self.connection_timeout).await {
                    Ok(conn) => {
                        debug!("Successfully obtained connection from {} pool at {}", $service_name, addr);
                        return Ok(conn);
                    }
                    Err(e) => {
                        let pool_state_after = pool.state().await;
                        warn!(
                            "{} connection pool at {} has no connection available. Error: {}, State before: {:?}, State after: {:?}",
                            $service_name,
                            addr,
                            e,
                            pool_state_before,
                            pool_state_after
                        );
                        return Err(CommonError::NoAvailableGrpcConnection(
                            $service_name.to_string(),
                            format!(
                                "get {} client failed, err: {}, state: {:?}",
                                $service_name,
                                e,
                                pool_state_after
                            ),
                        ));
                    }
                }
            }

            Err(CommonError::NoAvailableGrpcConnection(
                $service_name.to_string(),
                "connection pool is not initialized".to_string(),
            ))
        }
    };
}

#[derive(Clone)]
pub struct ClientPool {
    max_open_connection: u64,
    connection_timeout: Duration,
    // modules: meta service
    meta_service_inner_pools: DashMap<String, Pool<PlacementServiceManager>>,
    meta_service_journal_service_pools: DashMap<String, Pool<JournalServiceManager>>,
    meta_service_mqtt_service_pools: DashMap<String, Pool<MqttServiceManager>>,
    // modules: meta service service: leader cache
    meta_service_leader_addr_caches: DashMap<String, String>,

    // modules: broker mqtt
    broker_mqtt_grpc_pools: DashMap<String, Pool<BrokerMqttServiceManager>>,
    // modules: broker storage engine
    broker_storage_grpc_pools: DashMap<String, Pool<BrokerStorageServiceManager>>,
    // modules: broker common
    broker_common_grpc_pools: DashMap<String, Pool<BrokerCommonServiceManager>>,
}

impl ClientPool {
    pub fn new(max_open_connection: u64) -> Self {
        Self::new_with_timeout(
            max_open_connection,
            Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECS),
        )
    }

    pub fn new_with_timeout(max_open_connection: u64, connection_timeout: Duration) -> Self {
        Self {
            max_open_connection,
            connection_timeout,
            // modules: meta_service
            meta_service_inner_pools: DashMap::with_capacity(2),
            meta_service_journal_service_pools: DashMap::with_capacity(2),
            meta_service_mqtt_service_pools: DashMap::with_capacity(2),
            meta_service_leader_addr_caches: DashMap::with_capacity(2),
            // modules: mqtt_broker
            broker_mqtt_grpc_pools: DashMap::with_capacity(2),
            broker_storage_grpc_pools: DashMap::with_capacity(2),
            broker_common_grpc_pools: DashMap::with_capacity(2),
        }
    }

    // ----------modules: meta service -------------
    define_client_method!(
        meta_service_inner_services_client,
        meta_service_inner_pools,
        PlacementServiceManager,
        "PlacementServiceManager"
    );

    define_client_method!(
        meta_service_journal_services_client,
        meta_service_journal_service_pools,
        JournalServiceManager,
        "JournalServiceManager"
    );

    define_client_method!(
        meta_service_mqtt_services_client,
        meta_service_mqtt_service_pools,
        MqttServiceManager,
        "MqttServiceManager"
    );

    // ----------modules: mqtt broker -------------
    define_client_method!(
        mqtt_broker_mqtt_services_client,
        broker_mqtt_grpc_pools,
        BrokerMqttServiceManager,
        "BrokerMQTTServiceManager"
    );

    // ----------modules: storage broker -------------
    define_client_method!(
        broker_storage_services_client,
        broker_storage_grpc_pools,
        BrokerStorageServiceManager,
        "BrokerStorageServiceManager"
    );

    // ----------modules: common broker -------------
    define_client_method!(
        broker_common_services_client,
        broker_common_grpc_pools,
        BrokerCommonServiceManager,
        "BrokerCommonServiceManager"
    );

    // ----------leader cache management -------------
    pub fn get_leader_addr(&self, method: &str) -> Option<Ref<'_, String, String>> {
        self.meta_service_leader_addr_caches.get(method)
    }

    pub fn set_leader_addr(&self, method: String, leader_addr: String) {
        info!(
            "Update Leader cache for method={}, leader={}",
            method, leader_addr
        );
        self.meta_service_leader_addr_caches
            .insert(method, leader_addr);
    }

    pub fn clear_leader_cache(&self) {
        self.meta_service_leader_addr_caches.clear();
    }

    // ----------pool statistics -------------
    pub fn get_pool_count(&self) -> usize {
        self.meta_service_inner_pools.len()
            + self.meta_service_journal_service_pools.len()
            + self.meta_service_mqtt_service_pools.len()
            + self.broker_mqtt_grpc_pools.len()
    }

    // ----------connection pool warming -------------
    /// Warm up the MQTT Broker connection pool by pre-establishing connections
    /// This helps avoid timeout issues when the first request comes in
    pub async fn warmup_mqtt_broker_pool(&self, addr: &str) -> Result<(), CommonError> {
        info!("Warming up MQTT Broker connection pool for {}", addr);

        // Try to get and immediately return a connection to initialize the pool
        match self.mqtt_broker_mqtt_services_client(addr).await {
            Ok(conn) => {
                info!(
                    "Successfully warmed up MQTT Broker connection pool for {}",
                    addr
                );
                drop(conn); // Return connection to pool
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Failed to warm up MQTT Broker connection pool for {}: {}",
                    addr, e
                );
                Err(e)
            }
        }
    }

    /// Warm up multiple MQTT Broker connection pools concurrently
    pub async fn warmup_mqtt_broker_pools(
        &self,
        addrs: &[String],
    ) -> Vec<(String, Result<(), CommonError>)> {
        let mut results = Vec::new();

        for addr in addrs {
            let result = self.warmup_mqtt_broker_pool(addr).await;
            results.push((addr.clone(), result));
        }

        results
    }

    // ----------pool health monitoring -------------
    /// Get health status of MQTT Broker connection pool
    pub async fn get_mqtt_broker_pool_health(&self, addr: &str) -> Option<PoolHealthStatus> {
        if let Some(pool) = self.broker_mqtt_grpc_pools.get(addr) {
            let state = pool.state().await;
            Some(PoolHealthStatus {
                addr: addr.to_string(),
                max_open: state.max_open,
                connections: state.connections,
                in_use: state.in_use,
                idle: state.idle,
                is_healthy: state.connections > 0 && state.idle > 0,
            })
        } else {
            None
        }
    }

    /// Get health status of all MQTT Broker connection pools
    pub async fn get_all_mqtt_broker_pool_health(&self) -> Vec<PoolHealthStatus> {
        let mut statuses = Vec::new();

        for entry in self.broker_mqtt_grpc_pools.iter() {
            let addr = entry.key().clone();
            let pool = entry.value();
            let state = pool.state().await;

            statuses.push(PoolHealthStatus {
                addr,
                max_open: state.max_open,
                connections: state.connections,
                in_use: state.in_use,
                idle: state.idle,
                is_healthy: state.connections > 0,
            });
        }

        statuses
    }

    /// Clear unhealthy connections from MQTT Broker pool
    pub fn clear_mqtt_broker_pool(&self, addr: &str) -> bool {
        if self.broker_mqtt_grpc_pools.contains_key(addr) {
            self.broker_mqtt_grpc_pools.remove(addr);
            info!("Cleared MQTT Broker connection pool for {}", addr);
            true
        } else {
            false
        }
    }
}

/// Health status information for a connection pool
#[derive(Debug, Clone)]
pub struct PoolHealthStatus {
    pub addr: String,
    pub max_open: u64,
    pub connections: u64,
    pub in_use: u64,
    pub idle: u64,
    pub is_healthy: bool,
}
