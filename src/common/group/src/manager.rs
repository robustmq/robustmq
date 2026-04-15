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

use common_base::{error::common::CommonError, tools::now_second};
use common_config::broker::broker_config;
use common_metrics::storage_engine::{
    record_storage_engine_ops, record_storage_engine_ops_duration,
};
use dashmap::DashMap;
use grpc_clients::{meta::common::call::get_offset_data, pool::ClientPool};
use metadata_struct::adapter::adapter_offset::AdapterConsumerGroupOffset;
use protocol::meta::meta_service_common::GetOffsetDataRequest;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub(crate) struct LocalGroupData {
    pub tenant: String,
    pub group_name: String,
}

#[derive(Clone)]
pub struct OffsetManager {
    pub(crate) client_pool: Arc<ClientPool>,
    pub(crate) offset_info: DashMap<String, HashMap<String, u64>>,
    pub(crate) update_group_info: DashMap<String, LocalGroupData>,
}

impl OffsetManager {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        OffsetManager {
            client_pool,
            offset_info: DashMap::new(),
            update_group_info: DashMap::new(),
        }
    }

    // get consumer offset by group
    pub async fn get_offset(
        &self,
        tenant: &str,
        group: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, CommonError> {
        let start = std::time::Instant::now();
        let request = GetOffsetDataRequest {
            tenant: tenant.to_owned(),
            group: group.to_owned(),
        };
        let config = broker_config();
        let reply =
            get_offset_data(&self.client_pool, &config.get_meta_service_addr(), request).await?;

        let mut results = Vec::new();
        for raw in reply.offsets {
            results.push(AdapterConsumerGroupOffset {
                group: group.to_string(),
                shard_name: raw.shard_name,
                offset: raw.offset,
                ..Default::default()
            });
        }
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("get_offset_by_group");
        record_storage_engine_ops_duration("get_offset_by_group", duration_ms);
        Ok(results)
    }

    pub async fn commit_offset(
        &self,
        tenant: &str,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let key = self.key(tenant, group_name);
        if let Some(mut data) = self.offset_info.get_mut(&key) {
            for (shard_name, offset) in offset {
                data.insert(shard_name.to_string(), *offset);
            }
        } else {
            self.offset_info.insert(key.clone(), offset.clone());
        }

        self.update_group_info.insert(
            key,
            LocalGroupData {
                tenant: tenant.to_string(),
                group_name: group_name.to_string(),
            },
        );
        record_storage_engine_ops("commit_offset");
        Ok(())
    }

    pub fn heartbeat(&self, tenant: &str, group_name: &str) {
        if !now_second().is_multiple_of(60) {
            return;
        }
        let key = self.key(tenant, group_name);
        self.update_group_info.insert(
            key,
            LocalGroupData {
                tenant: tenant.to_string(),
                group_name: group_name.to_string(),
            },
        );
    }

    pub fn remove_group(&self, tenant: &str, group_name: &str) {
        let key = self.key(tenant, group_name);
        self.offset_info.remove(&key);
    }

    pub(crate) fn key(&self, tenant: &str, group_name: &str) -> String {
        format!("{}_{}", tenant, group_name)
    }
}
