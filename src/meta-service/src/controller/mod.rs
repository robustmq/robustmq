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

use crate::controller::connector_scheduler::ConnectorScheduler;
use crate::controller::engine_gc::start_engine_delete_gc_thread;
use crate::core::cache::MetaCacheManager;
use crate::raft::manager::MultiRaftManager;
use node_call::NodeCallManager;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tracing::error;

pub mod connector_scheduler;
pub mod connector_status;
pub mod engine_gc;

pub fn start_controller(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    call_manager: &Arc<NodeCallManager>,
    stop_send: Sender<bool>,
) {
    let mqtt_controller = BrokerController::new(
        raft_manager.clone(),
        cache_manager.clone(),
        call_manager.clone(),
    );
    tokio::spawn(async move {
        mqtt_controller.start(&stop_send).await;
    });
}

pub fn stop_controller(stop_send: Sender<bool>) {
    if let Err(e) = stop_send.send(true) {
        error!(
            "Failed to send stop signal, Failure to stop controller,Error message:{}",
            e
        );
    }
}

pub struct BrokerController {
    node_call_manager: Arc<NodeCallManager>,
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
}

impl BrokerController {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cache_manager: Arc<MetaCacheManager>,
        node_call_manager: Arc<NodeCallManager>,
    ) -> BrokerController {
        BrokerController {
            cache_manager,
            node_call_manager,
            raft_manager,
        }
    }

    pub async fn start(&self, stop_send: &broadcast::Sender<bool>) {
        // storage engine gc
        let raft_manager = self.raft_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let call_manager = self.node_call_manager.clone();
        let raw_stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            start_engine_delete_gc_thread(raft_manager, cache_manager, call_manager, raw_stop_send)
                .await;
        }));

        // connector manager
        let raft_manager = self.raft_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let call_manager = self.node_call_manager.clone();
        let raw_stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let scheduler = ConnectorScheduler::new(
                raft_manager.clone(),
                call_manager.clone(),
                cache_manager.clone(),
            );

            scheduler.run(&raw_stop_send).await;
        }));
    }
}
