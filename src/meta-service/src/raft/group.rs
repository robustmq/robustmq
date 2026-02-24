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

use std::{collections::HashMap, sync::Arc};

use common_metrics::meta::raft::{
    record_write_duration, record_write_failure, record_write_request, record_write_success,
};
use grpc_clients::pool::ClientPool;
use openraft::{raft::ClientWriteResponse, Raft};
use tokio::{
    sync::RwLock,
    time::{timeout, Instant},
};
use tracing::warn;

use crate::{
    core::error::MetaServiceError,
    raft::{
        manager::{MultiRaftManager, RaftStateMachineName, SLOW_RAFT_WRITE_WARN_THRESHOLD_MS},
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
    ) -> Result<Self, MetaServiceError> {
        let mut raft_group = HashMap::new();
        for i in 0..group_num {
            let key_name = RaftGroup::key_name(group_name, i);
            let raft_node = MultiRaftManager::create_raft_node(
                &RaftStateMachineName::METADATA,
                &client_pool,
                &rocksdb_engine_handler,
                &route,
            )
            .await?;
            raft_group.insert(key_name, raft_node);
        }

        Ok(RaftGroup {
            group_name: group_name.to_string(),
            raft_group,
            group_num,
            stop: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn write(
        &self,
        key: &str,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {

        let stop = self.stop.read().await;
        if *stop {
            return Err(MetaServiceError::RaftNodeHasStopped(
                RaftStateMachineName::METADATA.to_string(),
            ));
        }

        let machine = RaftStateMachineName::METADATA.as_str();
        let data_type = data.data_type.to_string();

        record_write_request(machine);
        let start = Instant::now();
        let write_timeout = MultiRaftManager::get_raft_write_timeout();

        let result = timeout(write_timeout, self.metadata_raft_node.client_write(data)).await;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_write_duration(machine, duration_ms);
        if duration_ms > SLOW_RAFT_WRITE_WARN_THRESHOLD_MS {
            warn!(
                "Raft write is slow. machine={}, data_type={}, duration_ms={:.2}",
                machine, data_type, duration_ms
            );
        }

        match result {
            Ok(Ok(response)) => {
                record_write_success(machine);
                Ok(Some(response))
            }
            Ok(Err(e)) => {
                record_write_failure(machine);
                warn!(
                    "Raft write failed. machine={}, data_type={}, duration_ms={:.2}, error={}",
                    machine, data_type, duration_ms, e
                );
                Err(e.into())
            }
            Err(_) => {
                record_write_failure(machine);
                warn!(
                    "Raft write timed out. machine={}, data_type={}, timeout={}s, duration_ms={:.2}",
                    machine, data_type, write_timeout.as_secs(), duration_ms
                );
                Err(MetaServiceError::CommonError(format!(
                    "Write metadata timeout after {}s, data_type={}",
                    write_timeout.as_secs(),
                    data_type
                )))
            }
        }
    }

    fn hash_num(&self, key: &str) -> String {}

    fn key_name(raft_name: &str, index: u32) -> String {
        format!("{}_{}", raft_name, index)
    }
}
