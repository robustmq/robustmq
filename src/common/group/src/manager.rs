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
use grpc_clients::{
    meta::common::call::{delete_offset_data, get_offset_data},
    pool::ClientPool,
};
use metadata_struct::adapter::adapter_offset::{AdapterCommitOffset, AdapterConsumerGroupOffset};
use protocol::meta::meta_service_common::{DeleteOffsetDataRequest, GetOffsetDataRequest};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub(crate) struct LocalGroupData {
    pub tenant: String,
    pub group_name: String,
}

#[derive(Clone)]
pub(crate) struct OffsetEntry {
    pub topic_name: String,
    pub partition: u32,
    pub offset: u64,
}

#[derive(Clone)]
pub struct OffsetManager {
    pub(crate) client_pool: Arc<ClientPool>,
    pub(crate) offset_info: DashMap<String, HashMap<String, OffsetEntry>>,
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

        // Check local cache first — commit_group_offset writes here synchronously,
        // so a FETCH immediately after an ACK on the same node sees the new offset
        // without waiting for the background flush to reach meta-service.
        let key = self.key(tenant, group);
        if let Some(cached) = self.offset_info.get(&key) {
            let results = cached
                .iter()
                .map(|(shard_name, entry)| AdapterConsumerGroupOffset {
                    group: group.to_string(),
                    shard_name: shard_name.clone(),
                    topic_name: entry.topic_name.clone(),
                    partition: entry.partition,
                    offset: entry.offset,
                    ..Default::default()
                })
                .collect();
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            record_storage_engine_ops("get_offset_by_group");
            record_storage_engine_ops_duration("get_offset_by_group", duration_ms);
            return Ok(results);
        }

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
                topic_name: raw.topic,
                partition: raw.partition,
                offset: raw.offset,
                ..Default::default()
            });
        }

        // Populate local cache so subsequent calls on this node avoid the RPC.
        if !results.is_empty() {
            let shard_map: HashMap<String, OffsetEntry> = results
                .iter()
                .map(|r| {
                    (
                        r.shard_name.clone(),
                        OffsetEntry {
                            topic_name: r.topic_name.clone(),
                            partition: r.partition,
                            offset: r.offset,
                        },
                    )
                })
                .collect();
            self.offset_info.insert(key, shard_map);
        }

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("get_offset_by_group");
        record_storage_engine_ops_duration("get_offset_by_group", duration_ms);
        Ok(results)
    }

    pub async fn commit_group_offset(
        &self,
        tenant: &str,
        group_name: &str,
        offsets: &[AdapterCommitOffset],
    ) -> Result<(), CommonError> {
        let key = self.key(tenant, group_name);
        let entries = offsets.iter().map(|o| {
            (
                o.shard_name.clone(),
                OffsetEntry {
                    topic_name: o.topic_name.clone(),
                    partition: o.partition,
                    offset: o.offset,
                },
            )
        });
        if let Some(mut data) = self.offset_info.get_mut(&key) {
            data.extend(entries);
        } else {
            self.offset_info.insert(key.clone(), entries.collect());
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

    /// Local-only counterpart to `remove_group`: drops just the given shards.
    pub fn remove_shards(&self, tenant: &str, group_name: &str, shard_names: &[String]) {
        let key = self.key(tenant, group_name);
        if let Some(mut data) = self.offset_info.get_mut(&key) {
            for shard_name in shard_names {
                data.remove(shard_name);
            }
        }
    }

    /// Deletes committed offsets for the given shards via meta-service, and
    /// clears them locally too so a same-node read can't race ahead of the
    /// cache-invalidation push meta-service sends to every node.
    pub async fn delete_group_offset(
        &self,
        tenant: &str,
        group_name: &str,
        shard_names: &[String],
    ) -> Result<(), CommonError> {
        let request = DeleteOffsetDataRequest {
            tenant: tenant.to_owned(),
            group: group_name.to_owned(),
            shard_names: shard_names.to_vec(),
        };
        let config = broker_config();
        delete_offset_data(&self.client_pool, &config.get_meta_service_addr(), request).await?;

        self.remove_shards(tenant, group_name, shard_names);
        Ok(())
    }

    pub(crate) fn key(&self, tenant: &str, group_name: &str) -> String {
        format!("{}_{}", tenant, group_name)
    }
}
