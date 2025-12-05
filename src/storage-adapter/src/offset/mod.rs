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

use crate::{
    offset::{cache::OffsetCacheManager, storage::OffsetStorageManager},
    storage::ShardOffset,
};
use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::loop_select_ticket,
};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;

pub mod cache;
pub mod storage;

#[derive(Clone)]
pub struct OffsetManager {
    enable_cache: bool,
    offset_storage: OffsetStorageManager,
    offset_cache_storage: OffsetCacheManager,
}

impl OffsetManager {
    pub fn new(client_pool: Arc<ClientPool>, rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let conf = broker_config();
        let offset_storage = OffsetStorageManager::new(client_pool.clone());
        let offset_cache_storage =
            OffsetCacheManager::new(rocksdb_engine_handler.clone(), client_pool.clone());
        OffsetManager {
            enable_cache: conf.storage_offset.enable_cache,
            offset_storage,
            offset_cache_storage,
        }
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        if self.enable_cache {
            self.offset_cache_storage
                .commit_offset(group_name, offset)
                .await
        } else {
            self.offset_storage.commit_offset(group_name, offset).await
        }
    }

    pub async fn flush(&self) -> Result<(), CommonError> {
        if self.enable_cache {
            return self.offset_cache_storage.flush().await;
        }
        Ok(())
    }

    pub async fn try_comparison_and_save_offset(&self) -> Result<(), CommonError> {
        if self.enable_cache {
            return self
                .offset_cache_storage
                .try_comparison_and_save_offset()
                .await;
        }
        Ok(())
    }

    pub async fn offset_save_thread(&self, stop_sx: broadcast::Sender<bool>) {
        if self.enable_cache {
            let ac_fn = async || -> ResultCommonError {
                self.offset_cache_storage
                    .async_commit_offset_to_storage()
                    .await
            };

            loop_select_ticket(ac_fn, 100, &stop_sx).await;
        }
    }

    pub async fn get_offset(&self, group: &str) -> Result<Vec<ShardOffset>, CommonError> {
        // If cache is enabled, flush pending updates before reading to ensure consistency
        if self.enable_cache {
            self.offset_cache_storage.flush().await?;
        }
        self.offset_storage.get_offset(group).await
    }
}
