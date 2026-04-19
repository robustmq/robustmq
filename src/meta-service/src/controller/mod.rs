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
use crate::controller::group_gc::start_group_gc_thread;
use crate::controller::mail_gc::start_email_gc_thread;
use crate::controller::topic_delete::start_topic_delete_thread;
use crate::core::cache::MetaCacheManager;
use crate::raft::manager::MultiRaftManager;
use broker_core::cache::NodeCacheManager;
use grpc_clients::pool::ClientPool;
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tracing::error;

pub mod connector_scheduler;
pub mod connector_status;
pub mod engine_gc;
pub mod group_gc;
pub mod mail_gc;
pub mod topic_delete;

pub fn start_controller(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    client_pool: &Arc<ClientPool>,
    node_cache: Arc<NodeCacheManager>,
    stop_send: Sender<bool>,
) {
    let mqtt_controller = BrokerController::new(
        raft_manager.clone(),
        cache_manager.clone(),
        call_manager.clone(),
        rocksdb_engine_handler.clone(),
        client_pool.clone(),
        node_cache,
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
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    node_cache: Arc<NodeCacheManager>,
}

impl BrokerController {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cache_manager: Arc<MetaCacheManager>,
        node_call_manager: Arc<NodeCallManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_pool: Arc<ClientPool>,
        node_cache: Arc<NodeCacheManager>,
    ) -> BrokerController {
        BrokerController {
            cache_manager,
            node_call_manager,
            raft_manager,
            rocksdb_engine_handler,
            client_pool,
            node_cache,
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

        // group offset gc
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let raft_manager = self.raft_manager.clone();
        let call_manager = self.node_call_manager.clone();
        let node_cache = self.node_cache.clone();
        let raw_stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            start_group_gc_thread(
                rocksdb_engine_handler,
                raft_manager,
                call_manager,
                node_cache,
                raw_stop_send,
            )
            .await;
        }));

        // email gc
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let raft_manager = self.raft_manager.clone();
        let call_manager = self.node_call_manager.clone();
        let raw_stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            start_email_gc_thread(
                rocksdb_engine_handler,
                raft_manager,
                call_manager,
                raw_stop_send,
            )
            .await;
        }));

        // topic delete gc
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let call_manager = self.node_call_manager.clone();
        let raft_manager = self.raft_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let client_pool = self.client_pool.clone();
        let raw_stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            start_topic_delete_thread(
                rocksdb_engine_handler,
                call_manager,
                raft_manager,
                cache_manager,
                client_pool,
                raw_stop_send,
            )
            .await;
        }));
    }
}
