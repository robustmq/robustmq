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

use std::{collections::HashMap, future::Future};
use common_base::errors::RobustMQError;
use mobc::Pool;
use placement_center::PlacementCenterConnectionManager;
use protocol::placement_center::placement::placement_center_service_client::PlacementCenterServiceClient;
use tonic::transport::Channel;
pub mod broker_server;
pub mod placement_center;
pub mod storage_engine;

pub struct ClientPool {
    placement_center_pools: HashMap<String, Pool<PlacementCenterConnectionManager>>,
}

impl Clone for ClientPool {
    fn clone(&self) -> Self {
        Self {
            placement_center_pools: self.placement_center_pools.clone(),
        }
    }
}

impl ClientPool {
    pub fn new() -> Self {
        let placement_center_pools = HashMap::new();
        Self {
            placement_center_pools,
        }
    }

    pub async fn get_placement_center_client(
        &mut self,
        addr: String,
    ) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
        if !self.placement_center_pools.contains_key(&addr) {
            let manager = PlacementCenterConnectionManager::new(addr.clone());
            let pool = Pool::builder().max_open(3).build(manager);
            self.placement_center_pools
                .insert(addr.clone(), pool.clone());
        }
        if let Some(client) = self.placement_center_pools.get(&addr) {
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

pub fn retry_times() -> u64 {
    return 16;
}

pub fn retry_sleep_time(times: u64) -> u64 {
    return times * 3;
}

pub async fn retry_call<T>(
    call: impl Future<Output = Result<T, RobustMQError>>,
) -> Result<T, RobustMQError> {
    return call.await;
}

#[cfg(test)]
mod tests {
    use protocol::placement_center::placement::RegisterNodeRequest;

    use crate::ClientPool;

    #[tokio::test]
    async fn conects() {
        let mut pool = ClientPool::new();
        let addr = "127.0.0.1:2193".to_string();
        match pool.get_placement_center_client(addr).await {
            Ok(mut client) => {
                let r = client.register_node(RegisterNodeRequest::default()).await;
            }
            Err(e) => {}
        }
    }
}
