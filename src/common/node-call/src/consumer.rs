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

use crate::handler::{send_delete_session_batch, send_last_will_batch, send_update_cache_batch};
use crate::{NodeCallData, BATCH_SIZE, WORKER_THREAD_NUM};
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

const WORKER_CHANNEL_SIZE: usize = BATCH_SIZE * 4;

pub fn start_node_consumer_thread(
    node: BrokerNode,
    client_pool: Arc<ClientPool>,
    receiver: mpsc::Receiver<NodeCallData>,
    stop_send: broadcast::Sender<bool>,
) {
    let worker_num = WORKER_THREAD_NUM;
    let mut worker_senders = Vec::with_capacity(worker_num);

    for i in 0..worker_num {
        let (tx, rx) = mpsc::channel(WORKER_CHANNEL_SIZE);
        worker_senders.push(tx);
        spawn_worker(
            i,
            node.clone(),
            client_pool.clone(),
            rx,
            stop_send.subscribe(),
        );
    }

    spawn_router(
        node.node_id,
        receiver,
        worker_senders,
        stop_send.subscribe(),
    );
}

fn spawn_router(
    node_id: u64,
    mut receiver: mpsc::Receiver<NodeCallData>,
    workers: Vec<mpsc::Sender<NodeCallData>>,
    mut stop_receiver: broadcast::Receiver<bool>,
) {
    let worker_num = workers.len();

    tokio::spawn(async move {
        info!(
            "Node consumer router started for node {}, workers={}",
            node_id, worker_num
        );

        loop {
            let data = tokio::select! {
                stop = stop_receiver.recv() => {
                    match stop {
                        Ok(true) | Err(broadcast::error::RecvError::Closed) => {
                            info!("Node consumer router received stop signal for node {}", node_id);
                            break;
                        }
                        _ => continue,
                    }
                }
                data = receiver.recv() => {
                    match data {
                        Some(msg) => msg,
                        None => {
                            info!("Node consumer router channel closed for node {}", node_id);
                            break;
                        }
                    }
                }
            };

            let idx = match data.partition_key() {
                None => 0,
                Some(key) => {
                    let mut hasher = DefaultHasher::new();
                    key.hash(&mut hasher);
                    (hasher.finish() as usize % (worker_num - 1)) + 1
                }
            };

            if workers[idx].send(data).await.is_err() {
                info!(
                    "Worker {} channel closed for node {}, stopping router",
                    idx, node_id
                );
                break;
            }
        }
    });
}

fn spawn_worker(
    worker_id: usize,
    node: BrokerNode,
    client_pool: Arc<ClientPool>,
    mut receiver: mpsc::Receiver<NodeCallData>,
    mut stop_receiver: broadcast::Receiver<bool>,
) {
    tokio::spawn(async move {
        info!(
            "Node consumer worker {} started for node {}",
            worker_id, node.node_id
        );

        loop {
            let first = tokio::select! {
                stop = stop_receiver.recv() => {
                    match stop {
                        Ok(true) | Err(broadcast::error::RecvError::Closed) => {
                            info!(
                                "Node consumer worker {} received stop signal for node {}",
                                worker_id, node.node_id
                            );
                            break;
                        }
                        _ => continue,
                    }
                }
                data = receiver.recv() => {
                    match data {
                        Some(msg) => msg,
                        None => {
                            info!(
                                "Node consumer worker {} channel closed for node {}",
                                worker_id, node.node_id
                            );
                            break;
                        }
                    }
                }
            };

            let mut batch = Vec::with_capacity(BATCH_SIZE);
            batch.push(first);
            while batch.len() < BATCH_SIZE {
                match receiver.try_recv() {
                    Ok(msg) => batch.push(msg),
                    Err(_) => break,
                }
            }

            dispatch_batch(&client_pool, &node.grpc_addr, batch).await;
        }
    });
}

async fn dispatch_batch(client_pool: &Arc<ClientPool>, addr: &str, batch: Vec<NodeCallData>) {
    let mut cache_updates = Vec::new();
    let mut delete_sessions = Vec::new();
    let mut last_will_messages = Vec::new();

    for msg in batch {
        match msg {
            NodeCallData::UpdateCache(data) => cache_updates.push(data),
            NodeCallData::DeleteSession(id) => delete_sessions.push(id),
            NodeCallData::SendLastWillMessage(item) => last_will_messages.push(item),
        }
    }

    if !cache_updates.is_empty() {
        send_update_cache_batch(client_pool, addr, &cache_updates).await;
    }

    if !delete_sessions.is_empty() {
        send_delete_session_batch(client_pool, addr, &delete_sessions).await;
    }

    if !last_will_messages.is_empty() {
        send_last_will_batch(client_pool, addr, &last_will_messages).await;
    }
}
