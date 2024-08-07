// Copyright 2023 RobustMQ Team
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
use crate::{
    cache::placement::PlacementCacheManager, raft::apply::RaftMachineApply,
    storage::rocksdb::RocksDBEngine,
};
use common_base::config::placement_center::placement_center_conf;
use std::sync::Arc;
use tokio::{select, sync::broadcast};

pub struct ClusterController {
    cluster_cache: Arc<PlacementCacheManager>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_send: broadcast::Sender<bool>,
}

impl ClusterController {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        placement_center_storage: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        stop_send: broadcast::Sender<bool>,
    ) -> ClusterController {
        let controller = ClusterController {
            cluster_cache,
            placement_center_storage,
            rocksdb_engine_handler,
            stop_send,
        };
        return controller;
    }

    // Start the heartbeat detection thread of the Storage Engine node
    pub async fn start_node_heartbeat_check(&self) {
        let mut stop_recv = self.stop_send.subscribe();
        let config = placement_center_conf();
        let mut heartbeat = BrokerHeartbeat::new(
            config.heartbeat_timeout_ms.into(),
            config.heartbeat_check_time_ms,
            self.cluster_cache.clone(),
            self.placement_center_storage.clone(),
        );
        loop {
            select! {
                val = stop_recv.recv() =>{
                    match val{
                        Ok(flag) => {
                            if flag {

                                break;
                            }
                        }
                        Err(_) => {}
                    }
                }
                _ = heartbeat.start()=>{

                }
            }
        }
    }
}
