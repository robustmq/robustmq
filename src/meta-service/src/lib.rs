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

#![allow(clippy::result_large_err)]
use crate::core::cache::{load_cache, MetaCacheManager};
use crate::core::controller::ClusterController;
use crate::raft::manager::MultiRaftManager;
use broker_core::cache::NodeCacheManager;
use common_base::task::{TaskKind, TaskSupervisor};
use common_config::broker::broker_config;
use delay_task::manager::DelayTaskManager;
use grpc_clients::pool::ClientPool;
use node_call::NodeCallManager;
use raft::leadership::monitoring_leader_transition;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

pub mod controller;
pub mod core;
pub mod raft;
pub mod server;
pub mod storage;

#[derive(Clone)]
pub struct MetaServiceServerParams {
    pub raft_manager: Arc<MultiRaftManager>,
    pub cache_manager: Arc<MetaCacheManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub client_pool: Arc<ClientPool>,
    pub node_call_manager: Arc<NodeCallManager>,
    pub delay_task_manager: Arc<DelayTaskManager>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
}
pub struct MetaServiceServer {
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    node_call_manager: Arc<NodeCallManager>,
    stop: broadcast::Sender<bool>,
    task_supervisor: Arc<TaskSupervisor>,
}

impl MetaServiceServer {
    pub fn new(
        params: MetaServiceServerParams,
        stop: broadcast::Sender<bool>,
    ) -> MetaServiceServer {
        MetaServiceServer {
            cache_manager: params.cache_manager,
            rocksdb_engine_handler: params.rocksdb_engine_handler,
            client_pool: params.client_pool,
            node_call_manager: params.node_call_manager,
            raft_manager: params.raft_manager,
            task_supervisor: params.task_supervisor,
            stop,
        }
    }

    pub async fn start(&mut self) {
        if let Err(e) = load_cache(&self.cache_manager, &self.rocksdb_engine_handler) {
            error!("Failed to load cache: {}", e);
            std::process::exit(1);
        }

        if let Err(e) = self.raft_manager.start().await {
            error!("Failed to start Raft manager: {}", e);
            std::process::exit(1);
        }

        self.start_background_services().await;

        self.awaiting_stop().await;
    }

    async fn start_background_services(&self) {
        // raft machine monitor
        let raft_manager = self.raft_manager.clone();
        let stop = self.stop.clone();
        self.task_supervisor
            .spawn(TaskKind::MetaRaftMachineMonitor.to_string(), async move {
                raft_manager.start_metrics_monitor(stop).await;
            });

        // monitor leader change
        let cache_manager = self.cache_manager.clone();
        let raft_manager = self.raft_manager.clone();
        let node_call_manager = self.node_call_manager.clone();
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let client_pool = self.client_pool.clone();
        let group_offset_expire_sec = broker_config().meta_runtime.group_offset_expire_sec;
        let stop = self.stop.clone();
        self.task_supervisor.spawn(
            TaskKind::MetaMonitorRaftLeaderChange.to_string(),
            async move {
                monitoring_leader_transition(
                    cache_manager,
                    raft_manager,
                    node_call_manager,
                    rocksdb_engine_handler,
                    client_pool,
                    group_offset_expire_sec,
                    stop,
                )
                .await;
            },
        );

        // broker node heartbeat check
        let ctrl = ClusterController::new(
            self.cache_manager.clone(),
            self.raft_manager.clone(),
            self.client_pool.clone(),
            self.node_call_manager.clone(),
        );
        let stop = self.stop.clone();
        self.task_supervisor.spawn(
            TaskKind::BrokerNodeHeartbeat.to_string(),
            Box::pin(async move {
                ctrl.start_node_heartbeat_check(&stop).await;
            }),
        );
    }

    pub async fn awaiting_stop(&self) {
        let raft_manager = self.raft_manager.clone();
        let mut recv = self.stop.subscribe();
        match recv.recv().await {
            Ok(_) => {
                info!("Meta service shutdown...");
                if let Err(e) = raft_manager.shutdown().await {
                    error!("Failed to shutdown Raft node: {}", e);
                } else {
                    info!("Raft node shutdown successfully.");
                }
            }
            Err(e) => {
                error!("Failed to receive stop signal: {}", e);
            }
        }
    }
}
