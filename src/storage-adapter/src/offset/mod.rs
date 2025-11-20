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

use crate::{offset::storage::OffsetStorageManager, storage::ShardOffset};
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use std::{collections::HashMap, sync::Arc};

pub mod cache;
pub mod storage;

pub struct OffsetManager {
    enable_cache: bool,
    offset_storage: OffsetStorageManager,
}

impl OffsetManager {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        let conf = broker_config();
        let offset_storage = OffsetStorageManager::new(client_pool.clone());
        OffsetManager {
            enable_cache: conf.storage_offset.enable_cache,
            offset_storage,
        }
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        namespace: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        if self.enable_cache {
        } else {
            return self
                .offset_storage
                .commit_offset(group_name, namespace, offset)
                .await;
        }
        Ok(())
    }

    pub async fn get_offset(&self, group: &str) -> Result<Vec<ShardOffset>, CommonError> {
        // Because the frequency of Get Offset is relatively low, to prevent inconsistent offset data
        // each time we obtain the offset, we need to retrieve it from the meta service.
        return self.offset_storage.get_offset(group).await;
    }
}
