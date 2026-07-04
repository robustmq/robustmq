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
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use common_metrics::meta::raft::{init_raft_shards_metrics, record_raft_apply_lag};
use grpc_clients::meta::common::call::join_cluster;
use grpc_clients::pool::ClientPool;
use openraft::raft::ClientWriteResponse;
use openraft::{Config, Raft, SnapshotPolicy};
use protocol::meta::meta_service_common::JoinClusterRequest;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

pub const DEFAULT_RAFT_WRITE_TIMEOUT_SEC: u64 = 30;
pub const SLOW_RAFT_WRITE_WARN_THRESHOLD_MS: f64 = 5.0;
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
        init_raft_shards_metrics(meta_rt.offset_raft_group_num, meta_rt.data_raft_group_num);

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

        let conf = broker_config();
        let self_id: u64 = conf.broker_id;
        let self_addr = conf
            .meta_addrs
            .get(&self_id.to_string())
            .map(|a| a.to_string().replace('"', ""))
            .ok_or_else(|| {
                CommonError::CommonError(format!("broker_id {} not found in meta_addrs", self_id))
            })?;

        let peers: Vec<(u64, String)> = conf
            .meta_addrs
            .iter()
            .filter_map(|(id_str, addr)| {
                let id: u64 = id_str.parse().ok()?;
                if id == self_id {
                    return None;
                }
                Some((id, addr.to_string().replace('"', "")))
            })
            .collect();

        self.metadata.start_nodes().await?;
        self.offset.start_nodes().await?;
        self.data.start_nodes().await?;

        // A node with persisted state just restarts — openraft re-establishes
        // replication on its own, no add_learner/change_membership needed.
        let initialized = match self.all_shards().next() {
            Some((_, raft)) => raft.is_initialized().await.unwrap_or(false),
            None => false,
        };

        if initialized {
            info!(
                "Node {} has persisted state, recovering existing cluster",
                self_id
            );
            info!("Multi-Raft started");
            return Ok(());
        }

        match Self::find_reachable_peer(&peers).await {
            // Fresh node, a peer is reachable: join the existing cluster.
            Some((peer_id, ref peer_addr)) => {
                info!("Joining cluster via peer {} at {}", peer_id, peer_addr);
                let req = JoinClusterRequest {
                    node_id: self_id,
                    rpc_addr: self_addr.clone(),
                };
                join_cluster(
                    &ClientPool::new(conf.runtime.channels_per_address),
                    &[peer_addr.as_str()],
                    req,
                )
                .await
                .map_err(|e| {
                    CommonError::CommonError(format!(
                        "Failed to join cluster via peer {}: {}",
                        peer_addr, e
                    ))
                })?;
                info!("Successfully joined cluster via peer {}", peer_addr);
            }
            // Fresh node, no peer reachable: first node, bootstrap single-node.
            None => {
                info!(
                    "No reachable peers, bootstrapping single-node cluster (node {})",
                    self_id
                );
                self.metadata
                    .bootstrap_single_node(self_id, &self_addr)
                    .await?;
                self.offset
                    .bootstrap_single_node(self_id, &self_addr)
                    .await?;
                self.data.bootstrap_single_node(self_id, &self_addr).await?;
            }
        }

        info!("Multi-Raft started");
        Ok(())
    }

    /// Iterate over all (shard_name, raft_node) pairs across every group.
    pub fn all_shards(&self) -> impl Iterator<Item = (&String, &Raft<TypeConfig>)> {
        self.metadata
            .all_nodes()
            .chain(self.offset.all_nodes())
            .chain(self.data.all_nodes())
    }

    pub fn is_metadata_leader(&self) -> bool {
        let shard_name = format!("{}_0", RaftStateMachineName::METADATA.as_str());
        let Some(node) = self.metadata.get_node(&shard_name) else {
            return false;
        };
        let m = node.metrics().borrow().clone();
        m.current_leader == Some(m.id)
    }

    /// The node id of the current metadata-raft leader, or None if no leader is elected.
    pub fn metadata_leader(&self) -> Option<u64> {
        let shard_name = format!("{}_0", RaftStateMachineName::METADATA.as_str());
        let node = self.metadata.get_node(&shard_name)?;
        node.metrics().borrow().current_leader
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

    pub async fn start_metrics_monitor(&self, stop_send: broadcast::Sender<bool>) {
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

    /// Try each peer address with a short TCP connect timeout.
    /// Returns the first reachable (node_id, addr) pair, or None.
    async fn find_reachable_peer(peers: &[(u64, String)]) -> Option<(u64, String)> {
        for (id, addr) in peers {
            match tokio::time::timeout(
                Duration::from_millis(500),
                TcpStream::connect(addr.as_str()),
            )
            .await
            {
                Ok(Ok(_)) => {
                    return Some((*id, addr.clone()));
                }
                Ok(Err(e)) => {
                    warn!("Peer {} at {} not reachable: {}", id, addr, e);
                }
                Err(_) => {
                    warn!("Peer {} at {} connection timed out", id, addr);
                }
            }
        }
        None
    }

    pub async fn create_raft_node(
        shard_name: &str,
        client_pool: &Arc<ClientPool>,
        rocksdb_engine_handler: &Arc<rocksdb_engine::rocksdb::RocksDBEngine>,
        route: &Arc<DataRoute>,
    ) -> Result<Raft<TypeConfig>, CommonError> {
        let config = Config {
            heartbeat_interval: 500,
            election_timeout_min: 10000,
            election_timeout_max: 20000,
            // Build a snapshot every 100 applied logs and keep a small log tail
            // afterwards. Without an active snapshot policy, openraft purges logs
            // while the persisted snapshot lags behind last_applied, so on restart
            // purge_upto ends up greater than snapshot_last_log_id and RaftCore
            // panics ("invalid state"). A modest threshold keeps snapshot and
            // applied state in sync across restarts.
            snapshot_policy: SnapshotPolicy::LogsSinceLast(100),
            max_in_snapshot_log_to_keep: 1000,
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
