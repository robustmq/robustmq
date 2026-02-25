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

use super::network::client::Network;
use super::store::new_storage;
use super::type_config::TypeConfig;
use crate::core::error::MetaServiceError;
use crate::raft::group::RaftGroup;
use crate::raft::route::data::StorageData;
use crate::raft::route::DataRoute;
use crate::raft::type_config::Node;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use common_metrics::meta::raft::record_raft_apply_lag;
use grpc_clients::pool::ClientPool;
use openraft::raft::ClientWriteResponse;
use openraft::{Config, Raft};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

pub const DEFAULT_RAFT_WRITE_TIMEOUT_SEC: u64 = 30;
pub const SLOW_RAFT_WRITE_WARN_THRESHOLD_MS: f64 = 1000.0;
type RaftShardNodes = Vec<(String, Raft<TypeConfig>)>;
type MetricsGroups = Vec<(String, RaftShardNodes)>;

#[derive(Clone, Debug)]
pub enum RaftStateMachineName {
    METADATA,
    OFFSET,
    DATA,
}

impl RaftStateMachineName {
    pub fn as_str(&self) -> &str {
        match self {
            RaftStateMachineName::METADATA => "metadata",
            RaftStateMachineName::OFFSET => "offset",
            RaftStateMachineName::DATA => "data",
        }
    }
}

impl std::str::FromStr for RaftStateMachineName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "metadata" | "meta" => Ok(RaftStateMachineName::METADATA),
            "offset" => Ok(RaftStateMachineName::OFFSET),
            "data" | "mqtt" => Ok(RaftStateMachineName::DATA),
            _ => {
                let group = s.rsplit_once('_').map(|(prefix, _)| prefix).unwrap_or(s);
                match group {
                    "metadata" | "meta" => Ok(RaftStateMachineName::METADATA),
                    "offset" => Ok(RaftStateMachineName::OFFSET),
                    "data" | "mqtt" => Ok(RaftStateMachineName::DATA),
                    _ => Err(format!("Invalid RaftStateMachineName: {}", s)),
                }
            }
        }
    }
}

impl std::fmt::Display for RaftStateMachineName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct MultiRaftManager {
    pub metadata: RaftGroup,
    pub offset: RaftGroup,
    pub data: RaftGroup,
    pub stop: Arc<RwLock<bool>>,
}

impl MultiRaftManager {
    pub async fn new(
        client_pool: Arc<ClientPool>,
        rocksdb_engine_handler: Arc<rocksdb_engine::rocksdb::RocksDBEngine>,
        route: Arc<DataRoute>,
    ) -> Result<Self, CommonError> {
        let conf = broker_config();
        let meta_rt = &conf.meta_runtime;

        info!(
            "Initializing Multi-Raft: metadata=1, offset={}, data={}",
            meta_rt.offset_raft_group_num, meta_rt.data_raft_group_num
        );
        let metadata = RaftGroup::new(
            "metadata",
            1,
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            route.clone(),
        )
        .await?;

        let offset = RaftGroup::new(
            "offset",
            meta_rt.offset_raft_group_num,
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            route.clone(),
        )
        .await?;

        let data = RaftGroup::new(
            "data",
            meta_rt.data_raft_group_num,
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            route.clone(),
        )
        .await?;

        info!("Multi-Raft initialized");
        Ok(MultiRaftManager {
            metadata,
            offset,
            data,
            stop: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<(), CommonError> {
        info!("Starting Multi-Raft");
        self.metadata.start().await?;
        self.offset.start().await?;
        self.data.start().await?;
        info!("Multi-Raft started");
        Ok(())
    }

    pub fn get_raft_node(&self, shard_name: &str) -> Result<&Raft<TypeConfig>, MetaServiceError> {
        if matches!(shard_name, "metadata" | "meta") {
            return self.metadata.get_node("metadata_0").ok_or_else(|| {
                MetaServiceError::CommonError("metadata_0 shard not found".to_string())
            });
        }
        if let Some(raft) = self.metadata.get_node(shard_name) {
            return Ok(raft);
        }
        if let Some(raft) = self.offset.get_node(shard_name) {
            return Ok(raft);
        }
        if let Some(raft) = self.data.get_node(shard_name) {
            return Ok(raft);
        }
        Err(MetaServiceError::CommonError(format!(
            "Unknown raft shard: {}",
            shard_name
        )))
    }

    pub fn get_raft_write_timeout() -> Duration {
        let conf = broker_config();
        Duration::from_secs(
            conf.meta_runtime
                .raft_write_timeout_sec
                .max(DEFAULT_RAFT_WRITE_TIMEOUT_SEC),
        )
    }

    pub async fn write_metadata(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        self.metadata.write("", data).await
    }

    pub async fn write_offset(
        &self,
        key: &str,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        self.offset.write(key, data).await
    }

    pub async fn write_data(
        &self,
        key: &str,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        self.data.write(key, data).await
    }

    pub fn start_metrics_monitor(&self, stop_send: broadcast::Sender<bool>) {
        let groups: MetricsGroups = [&self.metadata, &self.offset, &self.data]
            .iter()
            .map(|g| {
                let nodes: Vec<_> = g
                    .all_nodes()
                    .map(|(name, raft)| (name.clone(), raft.clone()))
                    .collect();
                (g.group_name.clone(), nodes)
            })
            .collect();

        tokio::spawn(async move {
            let mut stop_recv = stop_send.subscribe();
            let mut ticker = tokio::time::interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        for (_group_name, nodes) in &groups {
                            for (shard_name, node) in nodes {
                                let m = node.metrics().borrow().clone();
                                let last_log = m.last_log_index.unwrap_or(0);
                                let last_applied = m.last_applied.map(|l| l.index).unwrap_or(0);
                                record_raft_apply_lag(shard_name, last_log, last_applied);
                            }
                        }
                    }
                    val = stop_recv.recv() => {
                        if matches!(val, Ok(true) | Err(_)) { break; }
                    }
                }
            }
        });
    }

    pub async fn shutdown(&self) -> Result<(), CommonError> {
        let mut stop = self.stop.write().await;
        *stop = true;

        self.data
            .shutdown()
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))?;

        self.offset
            .shutdown()
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))?;

        self.metadata
            .shutdown()
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))?;

        Ok(())
    }

    pub async fn init_raft_node(
        shard_name: &str,
        raft_node: &Raft<TypeConfig>,
    ) -> Result<(), CommonError> {
        let conf = broker_config();
        let mut nodes = BTreeMap::new();
        for (node_id, addr) in conf.meta_addrs.clone() {
            let addr = addr.to_string().replace("\"", "");
            let node = Node {
                rpc_addr: addr.clone(),
                node_id: node_id.parse().unwrap(),
            };
            nodes.insert(node.node_id, node);
        }

        match raft_node.is_initialized().await {
            Ok(false) => {
                info!("[{}] Initializing with {} nodes", shard_name, nodes.len());
                raft_node.initialize(nodes.clone()).await.map_err(|e| {
                    CommonError::CommonError(format!(
                        "[{}] Failed to initialize Raft cluster: {}",
                        shard_name, e
                    ))
                })?;
                info!("[{}] Initialized", shard_name);
            }
            Ok(true) => {
                info!("[{}] Already initialized", shard_name);
            }
            Err(e) => {
                return Err(CommonError::CommonError(format!(
                    "[{}] Failed to check initialization status: {}",
                    shard_name, e
                )));
            }
        }

        Ok(())
    }

    pub async fn create_raft_node(
        shard_name: &str,
        client_pool: &Arc<ClientPool>,
        rocksdb_engine_handler: &Arc<rocksdb_engine::rocksdb::RocksDBEngine>,
        route: &Arc<DataRoute>,
    ) -> Result<Raft<TypeConfig>, CommonError> {
        let config = Config {
            heartbeat_interval: 100,
            election_timeout_min: 1000,
            election_timeout_max: 2000,
            ..Default::default()
        };

        let config = Arc::new(config.validate().map_err(|e| {
            CommonError::CommonError(format!(
                "[{}] Invalid Raft configuration: {}",
                shard_name, e
            ))
        })?);

        let conf = broker_config();

        let (log_store, state_machine_store) =
            new_storage(shard_name, rocksdb_engine_handler.clone(), route.clone()).await;

        let network = Network::new(shard_name.to_string(), client_pool.clone());

        match Raft::new(
            conf.broker_id,
            config.clone(),
            network,
            log_store,
            state_machine_store,
        )
        .await
        {
            Ok(raft_node) => {
                info!("[{}] Raft node ready", shard_name);
                Ok(raft_node)
            }
            Err(e) => Err(CommonError::CommonError(format!(
                "[{}] Failed to create Raft instance: {}",
                shard_name, e
            ))),
        }
    }
}
