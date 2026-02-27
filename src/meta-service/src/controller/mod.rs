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

use crate::controller::engine_gc::start_engine_delete_gc_thread;
use crate::core::cache::MetaCacheManager;
use crate::raft::manager::MultiRaftManager;
use grpc_clients::pool::ClientPool;
use node_call::NodeCallManager;
use std::sync::Arc;
use tokio::sync::broadcast;

pub mod connector;
pub mod engine_gc;

pub struct BrokerController {
    node_call_manager: Arc<NodeCallManager>,
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
    client_pool: Arc<ClientPool>,
}

impl BrokerController {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cache_manager: Arc<MetaCacheManager>,
        node_call_manager: Arc<NodeCallManager>,
        client_pool: Arc<ClientPool>,
    ) -> BrokerController {
        BrokerController {
            cache_manager,
            node_call_manager,
            raft_manager,
            client_pool,
        }
    }

    pub async fn start(&self, stop_send: &broadcast::Sender<bool>) {
        // storage engine gc
        let raft_manager = self.raft_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let call_manager = self.node_call_manager.clone();
        let client_pool = self.client_pool.clone();
        let raw_stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            start_engine_delete_gc_thread(
                raft_manager,
                cache_manager,
                call_manager,
                client_pool,
                raw_stop_send,
            )
            .await;
        }));
    }
}
