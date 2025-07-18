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

use super::heartbeat::BrokerHeartbeat;
use crate::controller::journal::call_node::JournalInnerCallManager;
use crate::controller::mqtt::call_broker::MQTTInnerCallManager;
use crate::core::cache::CacheManager;
use crate::raft::route::apply::StorageDriver;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

pub struct ClusterController {
    cluster_cache: Arc<CacheManager>,
    placement_center_storage: Arc<StorageDriver>,
    stop_send: broadcast::Sender<bool>,
    client_pool: Arc<ClientPool>,
    journal_call_manager: Arc<JournalInnerCallManager>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
}

impl ClusterController {
    pub fn new(
        cluster_cache: Arc<CacheManager>,
        placement_center_storage: Arc<StorageDriver>,
        stop_send: broadcast::Sender<bool>,
        client_pool: Arc<ClientPool>,
        journal_call_manager: Arc<JournalInnerCallManager>,
        mqtt_call_manager: Arc<MQTTInnerCallManager>,
    ) -> ClusterController {
        ClusterController {
            cluster_cache,
            placement_center_storage,
            stop_send,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
        }
    }

    // Start the heartbeat detection thread of the Storage Engine node
    pub async fn start_node_heartbeat_check(&self) {
        let mut stop_recv = self.stop_send.subscribe();
        let config = broker_config();
        let mut heartbeat = BrokerHeartbeat::new(
            config.place_runtime.heartbeat_timeout_ms,
            self.cluster_cache.clone(),
            self.placement_center_storage.clone(),
            self.client_pool.clone(),
            self.journal_call_manager.clone(),
            self.mqtt_call_manager.clone(),
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
                _ = heartbeat.start()=>{
                    sleep(Duration::from_millis(config.place_runtime.heartbeat_check_time_ms)).await;
                }
            }
        }
    }
}
