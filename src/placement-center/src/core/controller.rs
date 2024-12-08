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

use std::sync::Arc;

use common_base::config::placement_center::placement_center_conf;
use grpc_clients::pool::ClientPool;
use tokio::select;
use tokio::sync::broadcast;

use super::heartbeat::BrokerHeartbeat;
use crate::core::cache::PlacementCacheManager;
use crate::journal::controller::call_node::JournalInnerCallManager;
use crate::route::apply::RaftMachineApply;

pub struct ClusterController {
    cluster_cache: Arc<PlacementCacheManager>,
    placement_center_storage: Arc<RaftMachineApply>,
    stop_send: broadcast::Sender<bool>,
    client_pool: Arc<ClientPool>,
    call_manager: Arc<JournalInnerCallManager>,
}

impl ClusterController {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        placement_center_storage: Arc<RaftMachineApply>,
        stop_send: broadcast::Sender<bool>,
        client_pool: Arc<ClientPool>,
        call_manager: Arc<JournalInnerCallManager>,
    ) -> ClusterController {
        ClusterController {
            cluster_cache,
            placement_center_storage,
            stop_send,
            client_pool,
            call_manager,
        }
    }

    // Start the heartbeat detection thread of the Storage Engine node
    pub async fn start_node_heartbeat_check(&self) {
        let mut stop_recv = self.stop_send.subscribe();
        let config = placement_center_conf();
        let mut heartbeat = BrokerHeartbeat::new(
            config.heartbeat.heartbeat_timeout_ms,
            config.heartbeat.heartbeat_check_time_ms,
            self.cluster_cache.clone(),
            self.placement_center_storage.clone(),
            self.client_pool.clone(),
            self.call_manager.clone(),
        );
        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            break;
                        }
                    }
                }
                _ = heartbeat.start()=>{}
            }
        }
    }
}
