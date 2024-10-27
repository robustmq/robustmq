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

use std::sync::Arc;

use common_base::config::journal_server::journal_server_conf;
use dashmap::DashMap;
use grpc_clients::placement::kv::call::placement_set;
use grpc_clients::pool::ClientPool;
use log::{info, warn};
use protocol::placement_center::placement_center_kv::SetRequest;
use serde::{Deserialize, Serialize};

use super::error::JournalServerError;

fn key_name(cluster_name: &str, namespace: &str, shard_name: &str) -> String {
    format!("{}/{}/{}", cluster_name, namespace, shard_name)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Offset {
    pub namespace: String,
    pub shard_name: String,
    pub offset: u64,
}

pub struct OffsetManager {
    client_pool: Arc<ClientPool>,
    offsets: DashMap<String, Offset>,
}

impl OffsetManager {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        let offsets = DashMap::new();
        OffsetManager {
            client_pool,
            offsets,
        }
    }

    pub async fn commit_offset(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        offset: u64,
    ) -> Result<(), JournalServerError> {
        let key = key_name(cluster_name, namespace, shard_name);

        if let Some(current_offset) = self.offsets.get(&key) {
            if current_offset.offset < offset {
                warn!("Submitted Offset is x, which is less than the current offset y. namespace: {}, shard_name:{}",namespace,shard_name);
            }
        } else {
            info!("No Offset information in the cache, it could be the first commit offset, namespace: {}, shard_name:{}",namespace,shard_name);
        }

        let conf = journal_server_conf();
        let offset = Offset {
            namespace: namespace.to_string(),
            shard_name: shard_name.to_string(),
            offset,
        };

        let request = SetRequest {
            key: key.clone(),
            value: serde_json::to_string(&offset)?,
        };

        match placement_set(
            self.client_pool.clone(),
            conf.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                self.offsets.insert(key, offset);
            }
            Err(e) => {
                return Err(e.into());
            }
        }

        Ok(())
    }

    pub async fn get_offset(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Option<Offset> {
        let key = key_name(cluster_name, namespace, shard_name);
        if let Some(offset) = self.offsets.get(&key) {
            return Some(offset.clone());
        }
        None
    }
}
