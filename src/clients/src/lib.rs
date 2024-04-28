/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use common_base::errors::RobustMQError;
use mobc::Pool;
use placement::{
    journal::JournalServiceManager, kv::KvServiceManager, placement::PlacementServiceManager,
};
use protocol::placement_center::generate::{
    journal::engine_service_client::EngineServiceClient, kv::kv_service_client::KvServiceClient,
    placement::placement_center_service_client::PlacementCenterServiceClient,
};
use std::collections::HashMap;
use tonic::transport::Channel;

pub mod journal;
pub mod mqtt;
pub mod placement;

const MAX_RETRY_TIMES: usize = 16;
pub struct ClientPool {
    ip_max_num: u64,
    placement_service_pools: HashMap<String, Pool<PlacementServiceManager>>,
    engine_service_pools: HashMap<String, Pool<JournalServiceManager>>,
    kv_service_pools: HashMap<String, Pool<KvServiceManager>>,
}

impl Clone for ClientPool {
    fn clone(&self) -> Self {
        Self {
            ip_max_num: self.ip_max_num,
            placement_service_pools: self.placement_service_pools.clone(),
            engine_service_pools: self.engine_service_pools.clone(),
            kv_service_pools: self.kv_service_pools.clone(),
        }
    }
}

impl ClientPool {
    pub fn new(ip_max_num: u64) -> Self {
        let placement_center_pools = HashMap::new();
        let engine_service_pools = HashMap::new();
        let kv_service_pools = HashMap::new();
        Self {
            ip_max_num,
            placement_service_pools: placement_center_pools,
            engine_service_pools,
            kv_service_pools,
        }
    }

    pub async fn get_placement_services_client(
        &mut self,
        addr: String,
    ) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
        if !self.placement_service_pools.contains_key(&addr) {
            let manager = PlacementServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.placement_service_pools
                .insert(addr.clone(), pool.clone());
        }
        if let Some(client) = self.placement_service_pools.get(&addr) {
            match client.clone().get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection("".to_string()));
    }

    pub async fn get_engine_services_client(
        &mut self,
        addr: String,
    ) -> Result<EngineServiceClient<Channel>, RobustMQError> {
        if !self.engine_service_pools.contains_key(&addr) {
            let manager = JournalServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.engine_service_pools.insert(addr.clone(), pool.clone());
        }
        if let Some(client) = self.engine_service_pools.get(&addr) {
            match client.clone().get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection("".to_string()));
    }

    pub async fn get_kv_services_client(
        &mut self,
        addr: String,
    ) -> Result<KvServiceClient<Channel>, RobustMQError> {
        if !self.kv_service_pools.contains_key(&addr) {
            let manager = KvServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.kv_service_pools.insert(addr.clone(), pool.clone());
        }
        if let Some(client) = self.kv_service_pools.get(&addr) {
            match client.clone().get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection("".to_string()));
    }
}

pub fn retry_times() -> usize {
    return MAX_RETRY_TIMES;
}

pub fn retry_sleep_time(times: usize) -> u64 {
    return (times * 3) as u64;
}
