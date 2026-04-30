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

use broker_core::cache::NodeCacheManager;
use bytes::Bytes;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use futures::future::join_all;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use protocol::broker::broker::{BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use tokio::time::{timeout, Duration};

pub mod consumer;
pub mod dispatcher;
pub mod handler;

pub const GLOBAL_CHANNEL_SIZE: usize = 10000;
pub const NODE_CHANNEL_SIZE: usize = 5000;
pub const BATCH_SIZE: usize = 100;
pub const WORKER_THREAD_NUM: usize = 10;
pub const RPC_MAX_RETRIES: usize = 3;
pub const RPC_RETRY_BASE_MS: u64 = 50;

#[derive(Clone, Debug)]
pub struct UpdateCacheData {
    pub action_type: BrokerUpdateCacheActionType,
    pub resource_type: BrokerUpdateCacheResourceType,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum NodeCallData {
    UpdateCache(UpdateCacheData),
    SendLastWillMessage { tenant: String, client_id: String },
    GetQosData(String),
}

pub struct NodeCallRequest {
    pub data: NodeCallData,
    // Snapshot of nodes taken at send time; dispatcher uses this list to avoid a second lookup.
    pub nodes: Vec<BrokerNode>,
    // One slot per node; the dispatcher pops the matching sender by node index.
    pub reply_txs: Vec<Option<oneshot::Sender<Bytes>>>,
}

impl NodeCallData {
    pub fn partition_key(&self) -> Option<&str> {
        match self {
            NodeCallData::UpdateCache(_) => None,
            NodeCallData::SendLastWillMessage { client_id, .. } => Some(client_id.as_str()),
            NodeCallData::GetQosData(_) => None,
        }
    }
}

pub struct NodeCallManager {
    pub global_sender: RwLock<Option<mpsc::Sender<NodeCallRequest>>>,
    broker_cache: Arc<NodeCacheManager>,
    node_channels: Arc<DashMap<u64, mpsc::Sender<NodeCallRequest>>>,
    client_pool: Arc<ClientPool>,
}

impl NodeCallManager {
    pub fn new(client_pool: Arc<ClientPool>, broker_cache: Arc<NodeCacheManager>) -> Self {
        NodeCallManager {
            global_sender: RwLock::new(None),
            broker_cache,
            node_channels: Arc::new(DashMap::with_capacity(8)),
            client_pool,
        }
    }

    pub async fn send_with_reply(&self, data: NodeCallData) -> Result<Vec<Bytes>, CommonError> {
        let nodes = self.broker_cache.node_list();
        let node_count = nodes.len();

        let mut reply_txs = Vec::with_capacity(node_count);
        let mut reply_rxs = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            let (tx, rx) = oneshot::channel();
            reply_txs.push(Some(tx));
            reply_rxs.push(rx);
        }

        let request = NodeCallRequest {
            data,
            nodes,
            reply_txs,
        };

        {
            let read = self.global_sender.read().await;
            if let Some(sender) = read.as_ref() {
                sender.send(request).await.map_err(|e| {
                    CommonError::CommonError(format!("Failed to send to global channel: {}", e))
                })?;
            } else {
                return Err(CommonError::CommonError(
                    "NodeCallManager global sender is not initialized".to_string(),
                ));
            }
        }

        let replies = timeout(Duration::from_secs(5), join_all(reply_rxs))
            .await
            .map_err(|_| {
                CommonError::CommonError("send_with_reply timed out after 5s".to_string())
            })?;

        let results = replies.into_iter().flatten().collect();
        Ok(results)
    }

    pub async fn send(&self, data: NodeCallData) -> Result<(), CommonError> {
        let request = NodeCallRequest {
            data,
            nodes: Vec::new(),
            reply_txs: Vec::new(),
        };
        let read = self.global_sender.read().await;
        if let Some(sender) = read.as_ref() {
            sender.send(request).await.map_err(|e| {
                CommonError::CommonError(format!("Failed to send to global channel: {}", e))
            })?;
            return Ok(());
        }
        Err(CommonError::CommonError(
            "NodeCallManager global sender is not initialized; call start() before send()"
                .to_string(),
        ))
    }

    pub fn client_pool(&self) -> &Arc<ClientPool> {
        &self.client_pool
    }

    pub fn broker_cache(&self) -> &Arc<NodeCacheManager> {
        &self.broker_cache
    }

    /// Returns true once `start()` has initialised the global sender channel.
    /// Use this to wait for readiness before calling `send()`.
    pub async fn is_ready(&self) -> bool {
        self.global_sender.read().await.is_some()
    }

    pub async fn start(&self, stop_send: broadcast::Sender<bool>) {
        let (global_sender, global_receiver) = mpsc::channel(GLOBAL_CHANNEL_SIZE);
        {
            let mut write = self.global_sender.write().await;
            *write = Some(global_sender);
            // write guard dropped here, before the dispatcher loop starts
        }
        dispatcher::run(
            global_receiver,
            stop_send,
            self.node_channels.clone(),
            self.broker_cache.clone(),
            self.client_pool.clone(),
        )
        .await;
    }
}
