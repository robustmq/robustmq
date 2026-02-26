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

use crate::{consumer, NodeCallData, NodeChannel, NODE_CHANNEL_SIZE};
use broker_core::cache::BrokerCacheManager;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, warn};

pub async fn run(
    mut global_receiver: mpsc::Receiver<NodeCallData>,
    stop_send: broadcast::Sender<bool>,
    node_channels: Arc<DashMap<u64, NodeChannel>>,
    broker_cache: Arc<BrokerCacheManager>,
    client_pool: Arc<ClientPool>,
) {
    info!("NodeCallManager dispatcher started");
    let mut stop_receiver = stop_send.subscribe();

    loop {
        tokio::select! {
            maybe_data = global_receiver.recv() => {
                match maybe_data {
                    Some(data) => {
                        for node in broker_cache.node_list() {
                            let sender =
                                get_or_create_sender(&node_channels, &node, &client_pool, &stop_send);

                            if let Err(e) = sender.send(data.clone()).await {
                                warn!(
                                    "Failed to dispatch to node {}, removing channel: {}",
                                    node.node_id, e
                                );
                                remove_node_channel(&node_channels, node.node_id);
                            }
                        }
                    }
                    None => {
                        info!("Global channel closed, dispatcher stopping");
                        break;
                    }
                }
            }
            stop = stop_receiver.recv() => {
                match stop {
                    Ok(true) => {
                        info!("Received stop signal, dispatcher stopping");
                        break;
                    }
                    Ok(false) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Stop channel closed, dispatcher stopping");
                        break;
                    }
                }
            }
        };
    }

    let node_ids: Vec<u64> = node_channels.iter().map(|entry| *entry.key()).collect();
    for node_id in node_ids {
        remove_node_channel(&node_channels, node_id);
    }
    info!("NodeCallManager dispatcher exited");
}

fn get_or_create_sender(
    node_channels: &Arc<DashMap<u64, NodeChannel>>,
    node: &BrokerNode,
    client_pool: &Arc<ClientPool>,
    stop_send: &broadcast::Sender<bool>,
) -> mpsc::Sender<NodeCallData> {
    if let Some(entry) = node_channels.get(&node.node_id) {
        return entry.sender.clone();
    }

    let (sender, receiver) = mpsc::channel(NODE_CHANNEL_SIZE);
    let stop_receiver = stop_send.subscribe();
    consumer::start_node_consumer_thread(
        node.clone(),
        client_pool.clone(),
        receiver,
        stop_receiver,
    );
    node_channels.insert(
        node.node_id,
        NodeChannel {
            node: node.clone(),
            sender: sender.clone(),
        },
    );
    info!("Auto-created channel for node {}", node.node_id);
    sender
}

fn remove_node_channel(node_channels: &Arc<DashMap<u64, NodeChannel>>, node_id: u64) {
    node_channels.remove(&node_id);
}
