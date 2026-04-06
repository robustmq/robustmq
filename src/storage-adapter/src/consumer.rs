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

use crate::driver::StorageDriverManager;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use metadata_struct::storage::{adapter_read_config::AdapterReadConfig, record::StorageRecord};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub enum StartOffsetStrategy {
    Earliest,
    Latest,
    LatestPerSubject,
    ByStartOffset(u64),
    ByStartTime(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OffsetKey {
    tenant: String,
    topic: String,
    shard: String,
}

impl OffsetKey {
    fn new(tenant: &str, topic: &str, shard: &str) -> Self {
        OffsetKey {
            tenant: tenant.to_string(),
            topic: topic.to_string(),
            shard: shard.to_string(),
        }
    }
}

pub struct GroupConsumer {
    driver: Arc<StorageDriverManager>,
    group_name: String,
    /// Committed offsets: the starting offset for the next read per shard.
    current_offsets: DashMap<OffsetKey, u64>,
    /// Offsets advanced after the last next_messages call, not yet committed.
    pending_offsets: DashMap<OffsetKey, u64>,
    auto_commit: bool,
    start_offset_strategy: RwLock<StartOffsetStrategy>,
}

impl GroupConsumer {
    pub fn new(driver: Arc<StorageDriverManager>, group_name: impl Into<String>) -> Self {
        GroupConsumer {
            driver,
            group_name: group_name.into(),
            current_offsets: DashMap::new(),
            pending_offsets: DashMap::new(),
            auto_commit: true,
            start_offset_strategy: RwLock::new(StartOffsetStrategy::Earliest),
        }
    }

    pub fn new_manual(driver: Arc<StorageDriverManager>, group_name: impl Into<String>) -> Self {
        GroupConsumer {
            auto_commit: false,
            ..Self::new(driver, group_name)
        }
    }

    pub async fn set_start_offset_strategy(&self, strategy: StartOffsetStrategy) {
        let mut write = self.start_offset_strategy.write().await;
        *write = strategy;
    }

    pub async fn next_messages(
        &self,
        tenant: &str,
        topic_name: &str,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.ensure_offsets_loaded(tenant, topic_name).await?;

        let shard_offsets = self.current_shard_offsets(tenant, topic_name);
        let records = self
            .driver
            .read_by_offset(tenant, topic_name, &shard_offsets, read_config)
            .await?;

        self.after_read(tenant, topic_name, records).await
    }

    pub async fn next_messages_by_tags(
        &self,
        tenant: &str,
        topic_name: &str,
        tag: &str,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.ensure_offsets_loaded(tenant, topic_name).await?;

        let shard_offsets = self.current_shard_offsets(tenant, topic_name);
        let records = self
            .driver
            .read_by_tag(tenant, topic_name, tag, &shard_offsets, read_config)
            .await?;

        self.after_read(tenant, topic_name, records).await
    }

    /// Persist pending offsets to the offset store and advance current_offsets.
    /// Persist the pending offsets to the offset store and advance the internal consume position.
    ///
    /// After `next_messages` returns, the new offsets are staged in `pending_offsets`.
    /// Only when `commit` succeeds are they merged into `current_offsets`.
    /// If `commit` is not called, the next `next_messages` call re-reads the same batch,
    /// which is the desired behavior when message processing fails and a retry is needed.
    pub async fn commit(&self) -> Result<(), CommonError> {
        if self.pending_offsets.is_empty() {
            return Ok(());
        }

        let mut by_tenant_topic: HashMap<(String, String), HashMap<String, u64>> = HashMap::new();
        for e in self.pending_offsets.iter() {
            let key = e.key();
            by_tenant_topic
                .entry((key.tenant.clone(), key.topic.clone()))
                .or_default()
                .insert(key.shard.clone(), *e.value() + 1);
        }

        for ((tenant, _topic), shard_offsets) in &by_tenant_topic {
            self.driver
                .commit_offset(tenant, &self.group_name, shard_offsets)
                .await?;
        }

        // Only advance current_offsets after successful IO.
        for e in self.pending_offsets.iter() {
            self.current_offsets.insert(e.key().clone(), *e.value());
        }
        self.pending_offsets.clear();

        Ok(())
    }

    /// Merge pending offsets into current_offsets without persisting to the offset store.
    ///
    /// Moves the in-memory consume position forward to the end of the last read batch,
    /// so the next `next_messages` call reads new messages instead of the same batch.
    /// Unlike `commit`, no IO is performed, so the offset store is not updated and a
    /// restart will resume from the last `commit` position.
    pub fn advance(&self) {
        for e in self.pending_offsets.iter() {
            let mut cur = self.current_offsets.entry(e.key().clone()).or_insert(0);
            if *e.value() >= *cur {
                *cur = *e.value() + 1;
            }
        }
        self.pending_offsets.clear();
    }

    /// Collect per-shard read-start offsets for the given tenant+topic.
    /// Returns offset + 1 so the next read begins after the last consumed record.
    fn current_shard_offsets(&self, tenant: &str, topic_name: &str) -> HashMap<String, u64> {
        self.current_offsets
            .iter()
            .filter(|e| e.key().tenant == tenant && e.key().topic == topic_name)
            .map(|e| (e.key().shard.clone(), *e.value()))
            .collect()
    }

    /// Stage offsets from the returned records, then auto-commit if enabled.
    async fn after_read(
        &self,
        tenant: &str,
        topic_name: &str,
        records: Vec<StorageRecord>,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.stage_offsets(tenant, topic_name, &records);
        if self.auto_commit && !records.is_empty() {
            self.commit().await?;
        }
        Ok(records)
    }

    /// Record the highest offset seen per shard from the returned records.
    fn stage_offsets(&self, tenant: &str, topic_name: &str, records: &[StorageRecord]) {
        for record in records {
            let key = OffsetKey::new(tenant, topic_name, &record.metadata.shard);
            let offset = record.metadata.offset;
            let mut entry = self.pending_offsets.entry(key).or_insert(0);
            if offset > *entry {
                *entry = offset;
            }
        }
    }

    async fn ensure_offsets_loaded(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<(), CommonError> {
        let already_loaded = self
            .current_offsets
            .iter()
            .any(|e| e.key().tenant == tenant && e.key().topic == topic_name);

        if already_loaded {
            return Ok(());
        }

        let committed = self
            .driver
            .get_offset_by_group(tenant, &self.group_name)
            .await?;

        if !committed.is_empty() {
            for g in committed {
                self.current_offsets
                    .insert(OffsetKey::new(tenant, topic_name, &g.shard_name), g.offset);
            }
            return Ok(());
        }

        // No committed offset for this group — apply the start offset strategy.
        let shard_offsets = self.resolve_initial_offsets(tenant, topic_name).await?;

        for (shard_name, offset) in shard_offsets {
            self.current_offsets
                .insert(OffsetKey::new(tenant, topic_name, &shard_name), offset);
        }

        Ok(())
    }

    async fn resolve_initial_offsets(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<HashMap<String, u64>, CommonError> {
        let storage_list = self
            .driver
            .list_storage_resource(tenant, topic_name)
            .await?;
        let strategy = self.start_offset_strategy.read().await;
        match *strategy {
            StartOffsetStrategy::Earliest => {
                let offsets = storage_list
                    .into_values()
                    .map(|detail| (detail.shard_name, detail.offset.start_offset))
                    .collect();
                Ok(offsets)
            }
            StartOffsetStrategy::Latest => {
                let offsets = storage_list
                    .into_values()
                    .map(|detail| (detail.shard_name, detail.offset.end_offset))
                    .collect();
                Ok(offsets)
            }
            StartOffsetStrategy::LatestPerSubject => {
                let offsets = storage_list
                    .into_values()
                    .map(|detail| {
                        let offset = detail.offset.end_offset;
                        (detail.shard_name, offset)
                    })
                    .collect();
                Ok(offsets)
            }
            StartOffsetStrategy::ByStartOffset(start) => {
                let offsets = storage_list
                    .into_values()
                    .map(|detail| {
                        let offset = start
                            .max(detail.offset.start_offset)
                            .min(detail.offset.end_offset);
                        (detail.shard_name, offset)
                    })
                    .collect();
                Ok(offsets)
            }
            StartOffsetStrategy::ByStartTime(timestamp) => {
                let adapter_strategy = AdapterOffsetStrategy::Latest;
                let target = self
                    .driver
                    .get_offset_by_timestamp(tenant, topic_name, timestamp, adapter_strategy)
                    .await?;
                let offsets = storage_list
                    .into_values()
                    .map(|detail| {
                        let offset = target
                            .max(detail.offset.start_offset)
                            .min(detail.offset.end_offset);
                        (detail.shard_name, offset)
                    })
                    .collect();
                Ok(offsets)
            }
        }
    }
}
