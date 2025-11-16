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

use crate::{
    controller::{journal::StorageEngineController, mqtt::MqttController},
    core::cache::CacheManager,
    raft::manager::MultiRaftManager,
};
use grpc_clients::pool::ClientPool;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{sync::Arc, time::Duration};
use tokio::{
    select,
    sync::broadcast::{self, Sender},
    time::sleep,
};
use tracing::{error, info};

pub fn monitoring_leader_transition(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
    raft_manager: Arc<MultiRaftManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let mut metrics_rx = raft_manager.mqtt_raft_node.metrics();
    let mut controller_running = false;
    tokio::spawn(async move {
        let mut last_leader: Option<u64> = None;
        let mut stop_recv = stop_send.subscribe();
        let (controller_stop_recv, _) = broadcast::channel::<bool>(2);
        loop {
            select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        break;
                    }
                }
            }

            val =  metrics_rx.changed() => {
                match val {
                    Ok(_) => {
                        let mm = metrics_rx.borrow().clone();
                        if let Some(current_leader) = mm.current_leader {
                            if last_leader != Some(current_leader)  {
                                if mm.id == current_leader{
                                    info!("[mqtt] Leader transition has occurred. current leader is  {:?}. Previous leader was {:?}.mm id:{}", current_leader, last_leader, mm.id);
                                    start_controller(
                                        &rocksdb_engine_handler,
                                        &cache_manager,
                                        &client_pool,
                                        &raft_manager,
                                        controller_stop_recv.clone(),
                                    );
                                    controller_running = true;
                                } else if controller_running {
                                    stop_controller(controller_stop_recv.clone());
                                    controller_running = false
                                }

                                last_leader = Some(current_leader);
                            }
                        }
                    }

                    Err(changed_err) => {
                        error!("Error while watching metrics_rx: {}; quitting monitoring_leader_transition() loop",changed_err);}
                    }
                }
            }
        }
        sleep(Duration::from_secs(1)).await;
    });
}

pub fn start_controller(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    raft_manager: &Arc<MultiRaftManager>,
    stop_send: Sender<bool>,
) {
    let mqtt_controller = MqttController::new(
        rocksdb_engine_handler.clone(),
        cache_manager.clone(),
        client_pool.clone(),
        stop_send.clone(),
    );
    tokio::spawn(async move {
        mqtt_controller.start().await;
    });

    let journal_controller = StorageEngineController::new(
        raft_manager.clone(),
        cache_manager.clone(),
        client_pool.clone(),
        stop_send.clone(),
    );
    tokio::spawn(async move {
        journal_controller.start().await;
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
