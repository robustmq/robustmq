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

const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 3;

macro_rules! define_client_method {
    (
        $method_name:ident,
        $pool_field:ident,
        $manager:ty,
        $service_name:expr
    ) => {
        pub async fn $method_name(&self, addr: &str) -> Result<Connection<$manager>, CommonError> {
            if !self.$pool_field.contains_key(addr) {
                let manager = <$manager>::new(addr.to_owned());
                let pool = Pool::builder()
                    .max_open(self.max_open_connection)
                    .build(manager);
                self.$pool_field.insert(addr.to_owned(), pool);
            }

            if let Some(pool) = self.$pool_field.get(addr) {
                match pool.get_timeout(self.connection_timeout).await {
                    Ok(conn) => return Ok(conn),
                    Err(e) => {
                        return Err(CommonError::NoAvailableGrpcConnection(
                            $service_name.to_string(),
                            format!(
                                "get {} client failed, err: {}, state: {:?}",
                                $service_name,
                                e,
                                pool.state().await
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
    define_client_method!(
        meta_service_inner_services_client,
        meta_service_inner_pools,
        PlacementServiceManager,
        "PlacementService"
    );

    define_client_method!(
        meta_service_journal_services_client,
        meta_service_journal_service_pools,
        JournalServiceManager,
        "JournalService"
    );

    define_client_method!(
        meta_service_kv_services_client,
        meta_service_kv_service_pools,
        KvServiceManager,
        "KvService"
    );

    define_client_method!(
        meta_service_mqtt_services_client,
        meta_service_mqtt_service_pools,
        MqttServiceManager,
        "MqttService"
    );

    define_client_method!(
        meta_service_openraft_services_client,
        meta_service_openraft_service_pools,
        OpenRaftServiceManager,
        "OpenRaftService"
    );

    // ----------modules: mqtt broker -------------
    define_client_method!(
        mqtt_broker_mqtt_services_client,
        mqtt_broker_placement_service_pools,
        MqttBrokerPlacementServiceManager,
        "MQTTBrokerPlacementService"
    );

    // ----------modules: journal engine -------------
    define_client_method!(
        journal_inner_services_client,
        journal_inner_service_pools,
        JournalInnerServiceManager,
        "JournalInnerService"
    );

    define_client_method!(
        journal_admin_services_client,
        journal_admin_service_pools,
        JournalAdminServiceManager,
        "JournalAdminService"
    );

    // ----------leader cache management -------------
    pub fn get_leader_addr(&self, addr: &str) -> Option<Ref<'_, String, String>> {
        self.meta_service_leader_addr_caches.get(addr)
    }

    pub fn set_leader_addr(&self, addr: String, leader_addr: String) {
        info!(
            "Update the Leader information in the client cache with the new Leader address: {}",
            leader_addr
        );
        self.meta_service_leader_addr_caches
            .insert(addr, leader_addr);
    }

    pub fn clear_leader_cache(&self) {
        self.meta_service_leader_addr_caches.clear();
    }

    // ----------pool statistics -------------
    pub fn get_pool_count(&self) -> usize {
        self.meta_service_inner_pools.len()
            + self.meta_service_journal_service_pools.len()
            + self.meta_service_kv_service_pools.len()
            + self.meta_service_mqtt_service_pools.len()
            + self.meta_service_openraft_service_pools.len()
            + self.mqtt_broker_placement_service_pools.len()
            + self.journal_admin_service_pools.len()
            + self.journal_inner_service_pools.len()
    }
}
