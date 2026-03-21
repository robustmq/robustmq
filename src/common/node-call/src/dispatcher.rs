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

use crate::{consumer, NodeCallRequest, NODE_CHANNEL_SIZE};
use broker_core::cache::NodeCacheManager;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, warn};

pub async fn run(
    mut global_receiver: mpsc::Receiver<NodeCallRequest>,
    stop_send: broadcast::Sender<bool>,
    node_channels: Arc<DashMap<u64, mpsc::Sender<NodeCallRequest>>>,
    broker_cache: Arc<NodeCacheManager>,
    client_pool: Arc<ClientPool>,
) {
    let mut stop_receiver = stop_send.subscribe();

    loop {
        tokio::select! {
            maybe_data = global_receiver.recv() => {
                match maybe_data {
                    Some(mut request) => {
                        // Use the node snapshot bundled in the request (send_with_reply path) so
                        // that reply_txs slots stay aligned. Fall back to a live lookup for
                        // fire-and-forget calls where nodes is empty.
                        let nodes = if request.nodes.is_empty() {
                            broker_cache.node_list()
                        } else {
                            request.nodes.clone()
                        };

                        for (idx, node) in nodes.iter().enumerate() {
                            let sender =
                                get_or_create_sender(&node_channels, node, &client_pool, &stop_send);

                            // Extract the oneshot sender for this node; other slots remain None.
                            let reply_tx = request.reply_txs.get_mut(idx).and_then(|s| s.take());
                            let node_request = NodeCallRequest {
                                data: request.data.clone(),
                                nodes: Vec::new(),
                                reply_txs: vec![reply_tx],
                            };

                            if let Err(e) = sender.send(node_request).await {
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
    info!("Node call manager dispatcher exited");
}

fn get_or_create_sender(
    node_channels: &Arc<DashMap<u64, mpsc::Sender<NodeCallRequest>>>,
    node: &BrokerNode,
    client_pool: &Arc<ClientPool>,
    stop_send: &broadcast::Sender<bool>,
) -> mpsc::Sender<NodeCallRequest> {
    if let Some(entry) = node_channels.get(&node.node_id) {
        return entry.value().clone();
    }

    let (sender, receiver) = mpsc::channel(NODE_CHANNEL_SIZE);
    consumer::start_node_consumer_thread(
        node.clone(),
        client_pool.clone(),
        receiver,
        stop_send.clone(),
    );
    node_channels.insert(node.node_id, sender.clone());
    info!("Auto-created channel for node {}", node.node_id);
    sender
}

fn remove_node_channel(
    node_channels: &Arc<DashMap<u64, mpsc::Sender<NodeCallRequest>>>,
    node_id: u64,
) {
    node_channels.remove(&node_id);
}
