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

use crate::cache::OffsetCacheManager;
use crate::storage::OffsetStorageManager;
use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::loop_select_ticket,
};
use common_metrics::storage_engine::{
    record_storage_engine_ops, record_storage_engine_ops_duration, record_storage_engine_ops_fail,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::adapter_offset::AdapterConsumerGroupOffset;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tracing::error;

#[derive(Clone)]
pub struct OffsetManager {
    enable_cache: bool,
    offset_storage: OffsetStorageManager,
    offset_cache_storage: OffsetCacheManager,
}

impl OffsetManager {
    pub fn new(
        client_pool: Arc<ClientPool>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        enable_cache: bool,
    ) -> Self {
        let offset_storage = OffsetStorageManager::new(client_pool.clone());
        let offset_cache_storage = OffsetCacheManager::new(rocksdb_engine_handler, client_pool);
        OffsetManager {
            enable_cache,
            offset_storage,
            offset_cache_storage,
        }
    }

    pub async fn commit_offset(
        &self,
        tenant: &str,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let start = std::time::Instant::now();
        let result = if self.enable_cache {
            self.offset_cache_storage
                .commit_offset(tenant, group_name, offset)
                .await
        } else {
            self.offset_storage
                .commit_offset(tenant, group_name, offset)
                .await
        };
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("commit_offset");
        record_storage_engine_ops_duration("commit_offset", duration_ms);
        if result.is_err() {
            record_storage_engine_ops_fail("commit_offset");
        }
        result
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

    pub async fn offset_async_save_thread(&self, stop_sx: broadcast::Sender<bool>) {
        if self.enable_cache {
            let ac_fn = async || -> ResultCommonError {
                self.offset_cache_storage
                    .async_commit_offset_to_storage()
                    .await
            };

            loop_select_ticket(ac_fn, 100, &stop_sx).await;
        }
    }

    pub async fn get_offset(
        &self,
        tenant: &str,
        group: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, CommonError> {
        let start = std::time::Instant::now();
        // If cache is enabled, flush pending updates before reading to ensure consistency
        if self.enable_cache {
            self.offset_cache_storage.flush().await?;
        }
        let result = match self.offset_storage.get_offset(tenant, group).await {
            Ok(data) => Ok(data),
            Err(e) => {
                // Used for compatibility test cases.
                error!("{}", e);
                if e.to_string().contains("Call address list cannot be empty") {
                    return Ok(Vec::new());
                }
                Err(e)
            }
        };
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("get_offset_by_group");
        record_storage_engine_ops_duration("get_offset_by_group", duration_ms);
        if result.is_err() {
            record_storage_engine_ops_fail("get_offset_by_group");
        }
        result
    }
}
