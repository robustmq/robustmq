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

pub mod consumer;
pub mod dispatcher;
pub mod handler;

use broker_core::cache::BrokerCacheManager;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use protocol::broker::broker_common::{BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType};
use protocol::broker::broker_mqtt::LastWillMessageItem;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::info;

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
    DeleteSession(String),
    SendLastWillMessage(LastWillMessageItem),
}

impl NodeCallData {
    pub fn partition_key(&self) -> Option<&str> {
        match self {
            NodeCallData::UpdateCache(_) => None,
            NodeCallData::DeleteSession(client_id) => Some(client_id),
            NodeCallData::SendLastWillMessage(item) => Some(&item.client_id),
        }
    }
}

pub struct NodeCallManager {
    pub global_sender: RwLock<Option<mpsc::Sender<NodeCallData>>>,
    broker_cache: Arc<BrokerCacheManager>,
    node_channels: Arc<DashMap<u64, mpsc::Sender<NodeCallData>>>,
    client_pool: Arc<ClientPool>,
}

impl NodeCallManager {
    pub fn new(client_pool: Arc<ClientPool>, broker_cache: Arc<BrokerCacheManager>) -> Self {
        NodeCallManager {
            global_sender: RwLock::new(None),
            broker_cache,
            node_channels: Arc::new(DashMap::with_capacity(8)),
            client_pool,
        }
    }

    pub async fn send(&self, data: NodeCallData) -> Result<(), CommonError> {
        let read = self.global_sender.read().await;
        if let Some(sender) = read.clone() {
            sender.send(data).await.map_err(|e| {
                CommonError::CommonError(format!("Failed to send to global channel: {}", e))
            })?;
            return Ok(());
        }
        Err(CommonError::CommonError("".to_string()))
    }

    pub async fn start(&self, stop_send: broadcast::Sender<bool>) {
        let (global_sender, global_receiver) = mpsc::channel(GLOBAL_CHANNEL_SIZE);
        let mut write = self.global_sender.write().await;
        *write = Some(global_sender);

        tokio::spawn(dispatcher::run(
            global_receiver,
            stop_send,
            self.node_channels.clone(),
            self.broker_cache.clone(),
            self.client_pool.clone(),
        ));

        info!("Node call manager started");
    }
}
