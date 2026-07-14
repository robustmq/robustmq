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

use std::{
    collections::hash_map::DefaultHasher,
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use common_base::error::common::CommonError;
use common_metrics::meta::raft::{
    record_write_duration, record_write_failure, record_write_request, record_write_success,
};
use grpc_clients::pool::ClientPool;
use openraft::{raft::ClientWriteResponse, Raft};
use tokio::{
    sync::RwLock,
    time::{timeout, Instant},
};
use tracing::{debug, info, warn};

use crate::{
    core::error::MetaServiceError,
    raft::{
        manager::{MultiRaftManager, SLOW_RAFT_WRITE_WARN_THRESHOLD_MS},
        route::{data::StorageData, DataRoute},
        type_config::TypeConfig,
    },
};

pub struct RaftGroup {
    pub group_name: String,
    pub group_num: u32,
    pub raft_group: HashMap<String, Raft<TypeConfig>>,
    pub stop: Arc<RwLock<bool>>,
}

impl RaftGroup {
    pub async fn new(
        group_name: &str,
        group_num: u32,
        client_pool: Arc<ClientPool>,
        rocksdb_engine_handler: Arc<rocksdb_engine::rocksdb::RocksDBEngine>,
        route: Arc<DataRoute>,
    ) -> Result<Self, CommonError> {
        let group_num = group_num.max(1);
        let mut raft_group = HashMap::new();
        for i in 0..group_num {
            let shard_name = Self::shard_name(group_name, i);
            info!("Creating raft shard: {}", shard_name);
            let raft_node = MultiRaftManager::create_raft_node(
                &shard_name,
                &client_pool,
                &rocksdb_engine_handler,
                &route,
            )
            .await?;
            raft_group.insert(shard_name, raft_node);
        }

        Ok(RaftGroup {
            group_name: group_name.to_string(),
            raft_group,
            group_num,
            stop: Arc::new(RwLock::new(false)),
        })
    }

    /// Only check initialization status; log and return — join/bootstrap is
    /// handled once at the MultiRaftManager level.
    pub async fn start_nodes(&self) -> Result<(), CommonError> {
        for (shard_name, raft) in &self.raft_group {
            match raft.is_initialized().await {
                Ok(true) => {
                    info!("[{}] Already initialized, rejoining cluster", shard_name);
                }
                Ok(false) => {
                    info!(
                        "[{}] Not yet initialized, waiting for bootstrap or join",
                        shard_name
                    );
                }
                Err(e) => {
                    return Err(CommonError::CommonError(format!(
                        "[{}] Failed to check initialization status: {}",
                        shard_name, e
                    )));
                }
            }
        }
        Ok(())
    }

    /// Bootstrap every shard in this group as a single-node cluster.
    pub async fn bootstrap_single_node(
        &self,
        node_id: u64,
        rpc_addr: &str,
    ) -> Result<(), CommonError> {
        use crate::raft::type_config::Node;
        use std::collections::BTreeMap;

        for (shard_name, raft) in &self.raft_group {
            if raft.is_initialized().await.unwrap_or(true) {
                continue;
            }
            let mut nodes = BTreeMap::new();
            nodes.insert(
                node_id,
                Node {
                    rpc_addr: rpc_addr.to_string(),
                    node_id,
                },
            );
            raft.initialize(nodes).await.map_err(|e| {
                CommonError::CommonError(format!(
                    "[{}] Failed to bootstrap single-node cluster: {}",
                    shard_name, e
                ))
            })?;
            info!("[{}] Single-node cluster bootstrapped", shard_name);
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), MetaServiceError> {
        let mut stop = self.stop.write().await;
        *stop = true;

        for (name, raft) in &self.raft_group {
            raft.shutdown().await.map_err(|e| {
                MetaServiceError::CommonError(format!("Failed to stop raft {}: {}", name, e))
            })?;
        }
        Ok(())
    }

    pub async fn write(
        &self,
        key: &str,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        let stop = self.stop.read().await;
        if *stop {
            return Err(MetaServiceError::RaftNodeHasStopped(
                self.group_name.clone(),
            ));
        }

        let shard = self.route_shard(key);
        let data_type = data.data_type.to_string();
        let write_timeout = MultiRaftManager::get_raft_write_timeout();

        let raft = self.raft_group.get(&shard).ok_or_else(|| {
            MetaServiceError::CommonError(format!("Raft shard not found: {}", shard))
        })?;
        record_write_request(&shard);
        let start = Instant::now();
        let result = timeout(write_timeout, raft.client_write(data)).await;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_write_duration(&shard, duration_ms);
        if duration_ms > SLOW_RAFT_WRITE_WARN_THRESHOLD_MS {
            debug!(
                "Raft write is slow. shard={}, data_type={}, duration_ms={:.2}",
                shard, data_type, duration_ms
            );
        }

        match result {
            Ok(Ok(response)) => {
                record_write_success(&shard);
                Ok(Some(response))
            }
            Ok(Err(e)) => {
                record_write_failure(&shard);
                let e_str = e.to_string();
                if e_str.contains("has to forward request to") {
                    debug!(
                        "Raft write failed. shard={}, data_type={}, duration_ms={:.2}, error={}",
                        shard, data_type, duration_ms, e_str
                    );
                } else {
                    warn!(
                        "Raft write failed. shard={}, data_type={}, duration_ms={:.2}, error={}",
                        shard, data_type, duration_ms, e_str
                    );
                }
                Err(e.into())
            }
            Err(_) => {
                record_write_failure(&shard);
                warn!(
                    "Raft write timed out. shard={}, data_type={}, timeout={}s, duration_ms={:.2}",
                    shard,
                    data_type,
                    write_timeout.as_secs(),
                    duration_ms
                );
                Err(MetaServiceError::CommonError(format!(
                    "Write {} timeout after {}s, data_type={}",
                    self.group_name,
                    write_timeout.as_secs(),
                    data_type
                )))
            }
        }
    }

    pub fn get_node(&self, shard_name: &str) -> Option<&Raft<TypeConfig>> {
        self.raft_group.get(shard_name)
    }

    pub fn all_nodes(&self) -> impl Iterator<Item = (&String, &Raft<TypeConfig>)> {
        self.raft_group.iter()
    }

    fn route_shard(&self, key: &str) -> String {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let index = (hasher.finish() % self.group_num as u64) as u32;
        Self::shard_name(&self.group_name, index)
    }

    fn shard_name(group_name: &str, index: u32) -> String {
        format!("{}_{}", group_name, index)
    }
}
