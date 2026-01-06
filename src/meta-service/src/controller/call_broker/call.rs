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

use crate::core::error::MetaServiceError;
use broker_core::cache::BrokerCacheManager;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_base::utils::serialize;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use grpc_clients::broker::common::call::broker_common_update_cache;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use protocol::broker::broker_common::UpdateCacheRecord;
use protocol::broker::broker_common::UpdateCacheRequest;
use protocol::broker::broker_common::{BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

const BROKER_CALL_CHANNEL_SIZE: usize = 1000;
const BATCH_SIZE: usize = 100;
const BATCH_CHECK_INTERVAL_MS: u64 = 10;

pub async fn start_call_thread(
    node: BrokerNode,
    call_manager: Arc<BrokerCallManager>,
    client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
    ready_tx: oneshot::Sender<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Some(node_send) = call_manager.get_node_sender(node.node_id) {
            let mut stop_recv = stop_send.subscribe();
            let mut data_recv = node_send.sender.subscribe();

            if ready_tx.send(()).is_err() {
                warn!("Failed to notify thread ready for node {}", node.node_id);
            }

            info!(
                "Started inner communication thread between Meta Service and Broker node {}",
                node.node_id
            );

            loop {
                match stop_recv.try_recv() {
                    Ok(flag) => {
                        if flag {
                            info!(
                                "Stopped inner communication thread for Broker node {}",
                                node.node_id
                            );
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                        error!("Stop channel closed for node {}", node.node_id);
                        break;
                    }
                    Err(_) => {}
                }

                let mut batch = Vec::new();
                loop {
                    match data_recv.try_recv() {
                        Ok(msg) => {
                            batch.push(msg);
                            if batch.len() >= BATCH_SIZE {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::broadcast::error::TryRecvError::Lagged(skipped)) => {
                            warn!(
                                "Communication thread for node {} lagged, skipped {} messages",
                                node.node_id, skipped
                            );
                        }
                        Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                            info!(
                                "Data channel closed for node {}, stopping thread",
                                node.node_id
                            );
                            break;
                        }
                    }
                }

                if batch.is_empty() {
                    tokio::time::sleep(Duration::from_millis(BATCH_CHECK_INTERVAL_MS)).await;
                    continue;
                }

                let filtered: Vec<BrokerCallMessage> = batch
                    .into_iter()
                    .filter(|msg| !is_ignore_push(&node, msg))
                    .collect();

                if !filtered.is_empty() {
                    call_mqtt_update_cache(
                        &client_pool,
                        &call_manager.broker_cache,
                        &node.grpc_addr,
                        &filtered,
                    )
                    .await;
                }
            }
        } else {
            let _ = ready_tx.send(());
            warn!(
                "Failed to start communication thread for Broker node {}: sender not found",
                node.node_id
            );
        }
    })
}

fn is_ignore_push(node: &BrokerNode, data: &BrokerCallMessage) -> bool {
    if data.resource_type == BrokerUpdateCacheResourceType::Node {
        let broker_node = match serialize::deserialize::<BrokerNode>(&data.data) {
            Ok(node) => node,
            Err(e) => {
                error!(
                    "Failed to deserialize BrokerNode data for node {}: {}",
                    node.node_id, e
                );
                return true;
            }
        };
        return broker_node.node_id == node.node_id;
    }
    false
}

async fn call_mqtt_update_cache(
    client_pool: &Arc<ClientPool>,
    broker_cache: &Arc<BrokerCacheManager>,
    addr: &str,
    data: &[BrokerCallMessage],
) {
    let records = data
        .iter()
        .map(|raw| UpdateCacheRecord {
            action_type: raw.action_type.into(),
            resource_type: raw.resource_type.into(),
            data: raw.data.clone(),
        })
        .collect();

    let request = UpdateCacheRequest { records };
    if let Err(e) = broker_common_update_cache(client_pool, &[addr], request.clone()).await {
        if broker_cache.is_stop().await {
            return;
        }
        error!("Failed to update cache on Broker {},: {}", addr, e);
    };
}

pub async fn send_cache_update<F>(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    action_type: BrokerUpdateCacheActionType,
    resource_type: BrokerUpdateCacheResourceType,
    encode_fn: F,
) -> Result<(), MetaServiceError>
where
    F: FnOnce() -> Result<Vec<u8>, MetaServiceError>,
{
    let data = encode_fn()?;
    let message = BrokerCallMessage {
        action_type,
        resource_type,
        data,
    };
    add_call_message(call_manager, client_pool, message).await
}

pub async fn add_call_message(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    message: BrokerCallMessage,
) -> Result<(), MetaServiceError> {
    for raw in call_manager.broker_cache.node_list().iter() {
        match call_manager.node_sender.entry(raw.node_id) {
            Entry::Occupied(entry) => {
                entry.get().sender.send(message.clone()).map_err(|e| {
                    MetaServiceError::CommonError(format!(
                        "Failed to send message to Broker node {}: {}",
                        raw.node_id, e
                    ))
                })?;
            }
            Entry::Vacant(entry) => {
                let (sx, _) = broadcast::channel::<BrokerCallMessage>(BROKER_CALL_CHANNEL_SIZE);
                let node_sender = BrokerCallNodeSender {
                    sender: sx.clone(),
                    node: raw.clone(),
                };
                entry.insert(node_sender);

                let (stop_send, _) = broadcast::channel(2);
                call_manager.add_node_stop_sender(raw.node_id, stop_send.clone());

                let (ready_tx, ready_rx) = oneshot::channel();
                let handle = start_call_thread(
                    raw.clone(),
                    call_manager.clone(),
                    client_pool.clone(),
                    stop_send,
                    ready_tx,
                )
                .await;

                call_manager.node_thread_handle.insert(raw.node_id, handle);

                tokio::time::timeout(Duration::from_secs(1), ready_rx)
                    .await
                    .map_err(|_| {
                        MetaServiceError::CommonError(format!(
                            "Timeout waiting for thread to be ready for node {}",
                            raw.node_id
                        ))
                    })?
                    .map_err(|_| {
                        MetaServiceError::CommonError(format!(
                            "Failed to receive ready signal from thread for node {}",
                            raw.node_id
                        ))
                    })?;

                sx.send(message.clone()).map_err(|e| {
                    MetaServiceError::CommonError(format!(
                        "Failed to send message to Broker node {}: {}",
                        raw.node_id, e
                    ))
                })?;
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct BrokerCallMessage {
    pub action_type: BrokerUpdateCacheActionType,
    pub resource_type: BrokerUpdateCacheResourceType,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct BrokerCallNodeSender {
    pub sender: Sender<BrokerCallMessage>,
    pub node: BrokerNode,
}

pub struct BrokerCallManager {
    pub node_sender: DashMap<u64, BrokerCallNodeSender>,
    pub node_stop_sender: DashMap<u64, Sender<bool>>,
    pub node_thread_handle: DashMap<u64, JoinHandle<()>>,
    pub broker_cache: Arc<BrokerCacheManager>,
}

impl BrokerCallManager {
    pub fn new(broker_cache: Arc<BrokerCacheManager>) -> Self {
        let node_sender = DashMap::with_capacity(2);
        let node_stop_sender = DashMap::with_capacity(2);
        let node_thread_handle = DashMap::with_capacity(2);
        BrokerCallManager {
            node_sender,
            node_stop_sender,
            node_thread_handle,
            broker_cache,
        }
    }

    pub fn get_node_sender(&self, node_id: u64) -> Option<BrokerCallNodeSender> {
        if let Some(sender) = self.node_sender.get(&node_id) {
            return Some(sender.clone());
        }
        None
    }

    pub fn add_node_sender(&self, node_id: u64, sender: BrokerCallNodeSender) {
        self.node_sender.insert(node_id, sender);
    }

    pub async fn remove_node(&self, node_id: u64) {
        self.node_sender.remove(&node_id);

        if let Some((_, send)) = self.node_stop_sender.remove(&node_id) {
            if let Err(e) = send.send(true) {
                warn!(
                    "Failed to send stop signal to Broker node {}: {}",
                    node_id, e
                );
            }
        }

        if let Some((_, handle)) = self.node_thread_handle.remove(&node_id) {
            if let Err(e) = tokio::time::timeout(Duration::from_secs(5), handle).await {
                warn!(
                    "Timeout waiting for thread to stop for node {}: {}",
                    node_id, e
                );
            }
        }
    }

    pub fn add_node_stop_sender(&self, node_id: u64, sender: Sender<bool>) {
        self.node_stop_sender.insert(node_id, sender);
    }
}

pub async fn broker_call_thread_manager(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    stop: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        let nodes_need_thread: Vec<(u64, BrokerNode)> = call_manager
            .node_sender
            .iter()
            .filter_map(|entry| {
                let node_id = *entry.key();
                if !call_manager.node_stop_sender.contains_key(&node_id) {
                    Some((node_id, entry.value().node.clone()))
                } else {
                    None
                }
            })
            .collect();

        for (node_id, node) in nodes_need_thread {
            if let Entry::Vacant(entry) = call_manager.node_stop_sender.entry(node_id) {
                let (stop_send, _) = broadcast::channel(2);
                entry.insert(stop_send.clone());

                warn!(
                    "Detected orphaned sender for node {}, starting recovery thread",
                    node_id
                );

                let (ready_tx, _) = oneshot::channel();
                let handle = start_call_thread(
                    node,
                    call_manager.clone(),
                    client_pool.clone(),
                    stop_send,
                    ready_tx,
                )
                .await;

                call_manager.node_thread_handle.insert(node_id, handle);
            }
        }

        let stop_node_ids: Vec<(u64, broadcast::Sender<bool>)> = call_manager
            .node_stop_sender
            .iter()
            .filter(|entry| !call_manager.node_sender.contains_key(entry.key()))
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        for (node_id, sx) in stop_node_ids {
            if let Err(e) = sx.send(true) {
                error!(
                    "Failed to send stop signal to orphaned thread for node {}: {}",
                    node_id, e
                );
            }
            call_manager.node_stop_sender.remove(&node_id);
            call_manager.node_thread_handle.remove(&node_id);
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, 1000, &stop).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::broker::broker_common::{
        BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType,
    };
    use topic_mapping::manager::TopicManager;

    fn create_test_node(node_id: u64) -> BrokerNode {
        BrokerNode {
            roles: vec!["broker".to_string()],
            extend: vec![],
            node_id,
            node_ip: "127.0.0.1".to_string(),
            grpc_addr: format!("127.0.0.1:{}", 9000 + node_id),
            ..Default::default()
        }
    }

    #[test]
    fn test_is_ignore_push_non_node_type() {
        let node = create_test_node(1);
        let message = BrokerCallMessage {
            action_type: BrokerUpdateCacheActionType::Set,
            resource_type: BrokerUpdateCacheResourceType::Shard,
            data: vec![],
        };

        assert!(!is_ignore_push(&node, &message));
    }

    #[test]
    fn test_is_ignore_push_same_node_id() {
        let node = create_test_node(1);
        let target_node = create_test_node(1);
        let data = serialize::serialize(&target_node).unwrap();

        let message = BrokerCallMessage {
            action_type: BrokerUpdateCacheActionType::Set,
            resource_type: BrokerUpdateCacheResourceType::Node,
            data,
        };

        assert!(is_ignore_push(&node, &message));
    }

    #[test]
    fn test_is_ignore_push_different_node_id() {
        let node = create_test_node(1);
        let target_node = create_test_node(2);
        let data = serialize::serialize(&target_node).unwrap();

        let message = BrokerCallMessage {
            action_type: BrokerUpdateCacheActionType::Set,
            resource_type: BrokerUpdateCacheResourceType::Node,
            data,
        };

        assert!(!is_ignore_push(&node, &message));
    }

    #[test]
    fn test_is_ignore_push_invalid_data() {
        let node = create_test_node(1);
        let message = BrokerCallMessage {
            action_type: BrokerUpdateCacheActionType::Set,
            resource_type: BrokerUpdateCacheResourceType::Node,
            data: vec![1, 2, 3],
        };

        assert!(is_ignore_push(&node, &message));
    }

    #[test]
    fn test_manager_basic_operations() {
        let topic_manager = Arc::new(TopicManager::new());
        let broker_cache = Arc::new(BrokerCacheManager::new(
            common_config::broker::default_broker_config(),
            topic_manager,
        ));
        let manager = BrokerCallManager::new(broker_cache);

        let node = create_test_node(1);
        let (sx, _) = broadcast::channel::<BrokerCallMessage>(100);
        let node_sender = BrokerCallNodeSender {
            sender: sx,
            node: node.clone(),
        };

        assert!(manager.get_node_sender(1).is_none());

        manager.add_node_sender(1, node_sender.clone());

        let retrieved = manager.get_node_sender(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().node.node_id, 1);

        let (stop_sx, _) = broadcast::channel(2);
        manager.add_node_stop_sender(1, stop_sx);
        assert!(manager.node_stop_sender.contains_key(&1));
    }

    #[tokio::test]
    async fn test_thread_ready_signal() {
        let topic_manager = Arc::new(TopicManager::new());
        let broker_cache = Arc::new(BrokerCacheManager::new(
            common_config::broker::default_broker_config(),
            topic_manager,
        ));
        let manager = Arc::new(BrokerCallManager::new(broker_cache));
        let client_pool = Arc::new(ClientPool::new(1));

        let node = create_test_node(1);
        let (sx, _) = broadcast::channel::<BrokerCallMessage>(100);
        let node_sender = BrokerCallNodeSender {
            sender: sx.clone(),
            node: node.clone(),
        };
        manager.add_node_sender(1, node_sender);

        let (stop_send, _) = broadcast::channel(2);
        let (ready_tx, ready_rx) = oneshot::channel();

        let _handle =
            start_call_thread(node, manager.clone(), client_pool, stop_send, ready_tx).await;

        let result = tokio::time::timeout(Duration::from_secs(1), ready_rx).await;
        assert!(result.is_ok(), "Thread should send ready signal");
        assert!(result.unwrap().is_ok(), "Ready signal should be received");
    }

    #[tokio::test]
    async fn test_thread_stop() {
        let topic_manager = Arc::new(TopicManager::new());
        let broker_cache = Arc::new(BrokerCacheManager::new(
            common_config::broker::default_broker_config(),
            topic_manager,
        ));
        let manager = Arc::new(BrokerCallManager::new(broker_cache));
        let client_pool = Arc::new(ClientPool::new(1));

        let node = create_test_node(1);
        let (sx, _) = broadcast::channel::<BrokerCallMessage>(100);
        let node_sender = BrokerCallNodeSender {
            sender: sx,
            node: node.clone(),
        };
        manager.add_node_sender(1, node_sender);

        let (stop_send, _) = broadcast::channel(2);
        let (ready_tx, ready_rx) = oneshot::channel();

        let handle = start_call_thread(
            node,
            manager.clone(),
            client_pool,
            stop_send.clone(),
            ready_tx,
        )
        .await;

        ready_rx.await.unwrap();

        stop_send.send(true).unwrap();

        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "Thread should stop within timeout");
    }
}
