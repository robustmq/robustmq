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
use crate::{NodeCallData, BATCH_SIZE};
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

pub fn start_node_consumer_thread(
    node: BrokerNode,
    client_pool: Arc<ClientPool>,
    mut receiver: mpsc::Receiver<NodeCallData>,
    mut stop_receiver: broadcast::Receiver<bool>,
) {
    tokio::spawn(async move {
        info!("Node consumer started for node {}", node.node_id);

        loop {
            let first = tokio::select! {
                stop = stop_receiver.recv() => {
                    match stop {
                        Ok(true) => {
                            info!("Node consumer received stop signal for node {}", node.node_id);
                            break;
                        }
                        Ok(false) => {
                            continue;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("Node consumer stop channel closed for node {}", node.node_id);
                            break;
                        }
                    }
                }
                data = receiver.recv() => {
                    match data {
                        Some(msg) => msg,
                        None => {
                            info!("Node consumer exiting for node {}", node.node_id);
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

            let addr = &node.grpc_addr;
            dispatch_batch(&client_pool, addr, batch).await;
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
