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
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use metadata_struct::storage::{adapter_read_config::AdapterReadConfig, record::StorageRecord};
use std::{collections::HashMap, sync::Arc};

pub enum StartOffsetStrategy {
    Earliest,
    Latest,
    LatestPerSubject,
    ByStartOffset(u64),
    ByStartTime(u64),
}

pub struct GroupConsumer {
    driver: Arc<StorageDriverManager>,
    group_name: String,
    /// Committed offsets: the starting offset for the next read per shard.
    current_offsets: HashMap<(String, String, String), u64>,
    /// Offsets advanced after the last next_messages call, not yet committed.
    pending_offsets: HashMap<(String, String, String), u64>,
    auto_commit: bool,
    start_offset_strategy: StartOffsetStrategy,
}

impl GroupConsumer {
    pub fn new(driver: Arc<StorageDriverManager>, group_name: impl Into<String>) -> Self {
        GroupConsumer {
            driver,
            group_name: group_name.into(),
            current_offsets: HashMap::new(),
            pending_offsets: HashMap::new(),
            auto_commit: true,
            start_offset_strategy: StartOffsetStrategy::Earliest,
        }
    }

    pub fn new_manual(driver: Arc<StorageDriverManager>, group_name: impl Into<String>) -> Self {
        GroupConsumer {
            auto_commit: false,
            ..Self::new(driver, group_name)
        }
    }

    pub fn set_start_offset_strategy(&mut self, strategy: StartOffsetStrategy) {
        self.start_offset_strategy = strategy;
    }

    pub async fn next_messages(
        &mut self,
        tenant: &str,
        topic_name: &str,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.ensure_offsets_loaded(tenant, topic_name).await?;

        let shard_offsets: HashMap<String, u64> = self
            .current_offsets
            .iter()
            .filter(|((t, tp, _), _)| t == tenant && tp == topic_name)
            .map(|((_, _, shard), &offset)| (shard.clone(), offset))
            .collect();

        let records = self
            .driver
            .read_by_offset(tenant, topic_name, &shard_offsets, read_config)
            .await?;

        self.stage_offsets(tenant, topic_name, &records);

        if self.auto_commit && !records.is_empty() {
            self.commit().await?;
        }

        Ok(records)
    }

    /// Persist pending offsets to the offset store and advance current_offsets.
    /// Persist the pending offsets to the offset store and advance the internal consume position.
    ///
    /// After `next_messages` returns, the new offsets are staged in `pending_offsets`.
    /// Only when `commit` succeeds are they merged into `current_offsets`.
    /// If `commit` is not called, the next `next_messages` call re-reads the same batch,
    /// which is the desired behavior when message processing fails and a retry is needed.
    pub async fn commit(&mut self) -> Result<(), CommonError> {
        if self.pending_offsets.is_empty() {
            return Ok(());
        }

        let mut by_tenant_topic: HashMap<(&str, &str), HashMap<String, u64>> = HashMap::new();
        for ((tenant, topic, shard), &offset) in &self.pending_offsets {
            by_tenant_topic
                .entry((tenant.as_str(), topic.as_str()))
                .or_default()
                .insert(shard.clone(), offset);
        }

        for ((tenant, _topic), shard_offsets) in by_tenant_topic {
            self.driver
                .commit_offset(tenant, &self.group_name, &shard_offsets)
                .await?;
        }

        // Only advance current_offsets after successful IO.
        for (key, offset) in self.pending_offsets.drain() {
            self.current_offsets.insert(key, offset);
        }

        Ok(())
    }

    /// Merge pending offsets into current_offsets without persisting to the offset store.
    ///
    /// Moves the in-memory consume position forward to the end of the last read batch,
    /// so the next `next_messages` call reads new messages instead of the same batch.
    /// Unlike `commit`, no IO is performed, so the offset store is not updated and a
    /// restart will resume from the last `commit` position.
    pub fn advance(&mut self) {
        for (key, offset) in self.pending_offsets.drain() {
            let entry = self.current_offsets.entry(key).or_insert(0);
            if offset > *entry {
                *entry = offset;
            }
        }
    }

    fn stage_offsets(&mut self, tenant: &str, topic_name: &str, records: &[StorageRecord]) {
        for record in records {
            let key = (
                tenant.to_string(),
                topic_name.to_string(),
                record.metadata.shard.clone(),
            );
            let next = record.metadata.offset + 1;
            let entry = self.pending_offsets.entry(key).or_insert(0);
            if next > *entry {
                *entry = next;
            }
        }
    }

    async fn ensure_offsets_loaded(
        &mut self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<(), CommonError> {
        let already_loaded = self
            .current_offsets
            .keys()
            .any(|(t, tp, _)| t == tenant && tp == topic_name);

        if already_loaded {
            return Ok(());
        }

        let committed = self
            .driver
            .get_offset_by_group(tenant, &self.group_name)
            .await?;

        if !committed.is_empty() {
            for g in committed {
                self.current_offsets.insert(
                    (tenant.to_string(), topic_name.to_string(), g.shard_name),
                    g.offset,
                );
            }
            return Ok(());
        }

        // No committed offset for this group — apply the start offset strategy.
        let shard_offsets = self.resolve_initial_offsets(tenant, topic_name).await?;
        for (shard_name, offset) in shard_offsets {
            self.current_offsets.insert(
                (tenant.to_string(), topic_name.to_string(), shard_name),
                offset,
            );
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

        match &self.start_offset_strategy {
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
                    .map(|detail| {
                        (
                            detail.shard_name,
                            detail.offset.end_offset.saturating_add(1),
                        )
                    })
                    .collect();
                Ok(offsets)
            }
            StartOffsetStrategy::LatestPerSubject => {
                let offsets = storage_list
                    .into_values()
                    .map(|detail| {
                        let offset = detail.offset.end_offset.saturating_sub(1);
                        (detail.shard_name, offset)
                    })
                    .collect();
                Ok(offsets)
            }
            StartOffsetStrategy::ByStartOffset(start) => {
                let offsets = storage_list
                    .into_values()
                    .map(|detail| {
                        let offset = (*start)
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
                    .get_offset_by_timestamp(tenant, topic_name, *timestamp, adapter_strategy)
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
