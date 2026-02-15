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
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

const BROKER_CALL_CHANNEL_SIZE: usize = 5000;
const BATCH_SIZE: usize = 100;
const BATCH_CHECK_INTERVAL_MS: u64 = 10;
const UPDATE_CACHE_MAX_RETRIES: usize = 3;
const UPDATE_CACHE_RETRY_BASE_MS: u64 = 50;

pub async fn start_call_thread(
    node: BrokerNode,
    call_manager: Arc<BrokerCallManager>,
    client_pool: Arc<ClientPool>,
    mut data_recv: mpsc::Receiver<BrokerCallMessage>,
    mut stop_recv: mpsc::Receiver<bool>,
    ready_tx: oneshot::Sender<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
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
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    error!("Stop channel closed for node {}", node.node_id);
                    break;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
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
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        info!(
                            "Data channel closed for node {}, stopping communication thread",
                            node.node_id
                        );
                        return;
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
    for attempt in 1..=UPDATE_CACHE_MAX_RETRIES {
        match broker_common_update_cache(client_pool, &[addr], request.clone()).await {
            Ok(_) => return,
            Err(e) => {
                if broker_cache.is_stop().await {
                    return;
                }
                if attempt >= UPDATE_CACHE_MAX_RETRIES {
                    error!(
                        "Failed to update cache on Broker {} after {} attempts: {}",
                        addr, attempt, e
                    );
                    return;
                }
                warn!(
                    "Failed to update cache on Broker {} (attempt {}/{}): {}, retrying",
                    addr, attempt, UPDATE_CACHE_MAX_RETRIES, e
                );
                let backoff = UPDATE_CACHE_RETRY_BASE_MS.saturating_mul(1_u64 << (attempt - 1));
                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }
        }
    }
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
    for attempt in 1..=UPDATE_CACHE_MAX_RETRIES {
        match add_call_message(call_manager, client_pool, message.clone()).await {
            Ok(()) => return Ok(()),
            Err(MetaServiceError::RetryableNodeThreadRace(node_ids)) => {
                if attempt >= UPDATE_CACHE_MAX_RETRIES {
                    return Err(MetaServiceError::RetryableNodeThreadRace(node_ids));
                }
                warn!(
                    "Retryable node thread race on attempt {}/{} for node(s) [{}], retrying",
                    attempt, UPDATE_CACHE_MAX_RETRIES, node_ids
                );
                let backoff = UPDATE_CACHE_RETRY_BASE_MS.saturating_mul(1_u64 << (attempt - 1));
                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }
            Err(e) => return Err(e),
        }
    }

    Err(MetaServiceError::CommonError(
        "send_cache_update retry loop exited unexpectedly".to_string(),
    ))
}

pub async fn add_call_message(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    message: BrokerCallMessage,
) -> Result<(), MetaServiceError> {
    let mut errors = Vec::new();
    let mut retryable_race_nodes = Vec::new();
    for raw in call_manager.broker_cache.node_list().iter() {
        if let Some(existing) = call_manager.node_sender.get(&raw.node_id) {
            let sender = existing.sender.clone();
            drop(existing);
            if let Err(e) = sender.send(message.clone()).await {
                warn!("Node {} sender send failed: {}", raw.node_id, e);
                errors.push(format!(
                    "Failed to send message to Broker node {}: {}",
                    raw.node_id, e
                ));
            }
            continue;
        }

        // create node push thread task outside map guard
        let (data_sx, data_rx) = mpsc::channel::<BrokerCallMessage>(BROKER_CALL_CHANNEL_SIZE);
        let (stop_send, stop_recv) = mpsc::channel::<bool>(5);
        let (ready_tx, ready_rx) = oneshot::channel();

        let handle = start_call_thread(
            raw.clone(),
            call_manager.clone(),
            client_pool.clone(),
            data_rx,
            stop_recv,
            ready_tx,
        )
        .await;

        // wait ready
        if tokio::time::timeout(Duration::from_secs(5), ready_rx)
            .await
            .map_err(|_| {
                MetaServiceError::CommonError(format!(
                    "Timeout waiting for thread to be ready for node {}",
                    raw.node_id
                ))
            })
            .and_then(|v| {
                v.map_err(|_| {
                    MetaServiceError::CommonError(format!(
                        "Failed to receive ready signal from thread for node {}",
                        raw.node_id
                    ))
                })
            })
            .is_err()
        {
            errors.push(format!(
                "Failed to initialize communication thread for Broker node {}",
                raw.node_id
            ));
            continue;
        }

        // insert sender in short critical section
        let inserted = match call_manager.node_sender.entry(raw.node_id) {
            Entry::Vacant(vacant) => {
                let node_sender = BrokerCallNodeSender {
                    sender: data_sx.clone(),
                    node: raw.clone(),
                };
                vacant.insert(node_sender);
                true
            }
            Entry::Occupied(_) => false,
        };

        if !inserted {
            // Another task won the race and has installed a sender/thread.
            let _ = stop_send.send(true).await;
            let _ = tokio::time::timeout(Duration::from_secs(1), handle).await;
            retryable_race_nodes.push(raw.node_id);
            continue;
        }

        call_manager.node_thread_handle.insert(raw.node_id, handle);
        call_manager.add_node_stop_sender(raw.node_id, stop_send.clone());

        // send message
        if let Err(e) = data_sx.send(message.clone()).await {
            warn!("Node {} sender failed after insertion: {}", raw.node_id, e);
            errors.push(format!(
                "Failed to send message to Broker node {}: {}",
                raw.node_id, e
            ));
        }
    }
    if !errors.is_empty() {
        return Err(MetaServiceError::CommonError(errors.join("; ")));
    }
    if !retryable_race_nodes.is_empty() {
        let nodes = retryable_race_nodes
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        return Err(MetaServiceError::RetryableNodeThreadRace(nodes));
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
            if let Err(e) = send.send(true).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::{meta::node::BrokerNode, mqtt::node_extend::NodeExtend};
    use protocol::broker::broker_common::{
        BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType,
    };

    fn create_test_node(node_id: u64) -> BrokerNode {
        BrokerNode {
            roles: vec!["broker".to_string()],
            extend: NodeExtend::default(),
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
            action_type: BrokerUpdateCacheActionType::Create,
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
            action_type: BrokerUpdateCacheActionType::Create,
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
            action_type: BrokerUpdateCacheActionType::Create,
            resource_type: BrokerUpdateCacheResourceType::Node,
            data,
        };

        assert!(!is_ignore_push(&node, &message));
    }

    #[test]
    fn test_is_ignore_push_invalid_data() {
        let node = create_test_node(1);
        let message = BrokerCallMessage {
            action_type: BrokerUpdateCacheActionType::Create,
            resource_type: BrokerUpdateCacheResourceType::Node,
            data: vec![1, 2, 3],
        };

        assert!(is_ignore_push(&node, &message));
    }

    #[test]
    fn test_manager_basic_operations() {
        let broker_cache = Arc::new(BrokerCacheManager::new(
            common_config::broker::default_broker_config(),
        ));
        let manager = BrokerCallManager::new(broker_cache);

        let node = create_test_node(1);
        let (sx, _) = mpsc::channel::<BrokerCallMessage>(100);
        let node_sender = BrokerCallNodeSender {
            sender: sx,
            node: node.clone(),
        };

        assert!(manager.get_node_sender(1).is_none());

        manager.add_node_sender(1, node_sender.clone());

        let retrieved = manager.get_node_sender(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().node.node_id, 1);

        let (stop_sx, _) = mpsc::channel(2);
        manager.add_node_stop_sender(1, stop_sx);
        assert!(manager.node_stop_sender.contains_key(&1));
    }

    #[tokio::test]
    async fn test_thread_ready_signal() {
        let broker_cache = Arc::new(BrokerCacheManager::new(
            common_config::broker::default_broker_config(),
        ));
        let manager = Arc::new(BrokerCallManager::new(broker_cache));
        let client_pool = Arc::new(ClientPool::new(1));

        let node = create_test_node(1);
        let (sx, rx) = mpsc::channel::<BrokerCallMessage>(100);
        let node_sender = BrokerCallNodeSender {
            sender: sx.clone(),
            node: node.clone(),
        };
        manager.add_node_sender(1, node_sender);

        let (_, stop_recv) = mpsc::channel(2);
        let (ready_tx, ready_rx) = oneshot::channel();

        let _handle =
            start_call_thread(node, manager.clone(), client_pool, rx, stop_recv, ready_tx).await;

        let result = tokio::time::timeout(Duration::from_secs(1), ready_rx).await;
        assert!(result.is_ok(), "Thread should send ready signal");
        assert!(result.unwrap().is_ok(), "Ready signal should be received");
    }

    #[tokio::test]
    async fn test_thread_stop() {
        let broker_cache = Arc::new(BrokerCacheManager::new(
            common_config::broker::default_broker_config(),
        ));
        let manager = Arc::new(BrokerCallManager::new(broker_cache));
        let client_pool = Arc::new(ClientPool::new(1));

        let node = create_test_node(1);
        let (sx, rx) = mpsc::channel::<BrokerCallMessage>(100);
        let node_sender = BrokerCallNodeSender {
            sender: sx,
            node: node.clone(),
        };
        manager.add_node_sender(1, node_sender);

        let (stop_send, stop_recv) = mpsc::channel(2);
        let (ready_tx, ready_rx) = oneshot::channel();

        let handle =
            start_call_thread(node, manager.clone(), client_pool, rx, stop_recv, ready_tx).await;

        ready_rx.await.unwrap();

        stop_send.send(true).await.unwrap();

        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "Thread should stop within timeout");
    }
}
