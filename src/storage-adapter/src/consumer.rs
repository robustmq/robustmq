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
use metadata_struct::storage::{adapter_read_config::AdapterReadConfig, record::StorageRecord};
use std::{collections::HashMap, sync::Arc};

pub struct GroupConsumer {
    driver: Arc<StorageDriverManager>,
    group_name: String,
    current_offsets: HashMap<(String, String, String), u64>,
    auto_commit: bool,
}

impl GroupConsumer {
    pub fn new(driver: Arc<StorageDriverManager>, group_name: impl Into<String>) -> Self {
        GroupConsumer {
            driver,
            group_name: group_name.into(),
            current_offsets: HashMap::new(),
            auto_commit: true,
        }
    }

    pub fn new_manual(driver: Arc<StorageDriverManager>, group_name: impl Into<String>) -> Self {
        GroupConsumer {
            auto_commit: false,
            ..Self::new(driver, group_name)
        }
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

        self.advance_offsets(tenant, topic_name, &records);

        if self.auto_commit && !records.is_empty() {
            self.commit().await?;
        }

        Ok(records)
    }

    pub async fn commit(&self) -> Result<(), CommonError> {
        let mut by_tenant_topic: HashMap<(&str, &str), HashMap<String, u64>> = HashMap::new();
        for ((tenant, topic, shard), &offset) in &self.current_offsets {
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
        Ok(())
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

        if committed.is_empty() {
            self.current_offsets.insert(
                (tenant.to_string(), topic_name.to_string(), String::new()),
                0,
            );
        } else {
            for g in committed {
                self.current_offsets.insert(
                    (tenant.to_string(), topic_name.to_string(), g.shard_name),
                    g.offset,
                );
            }
        }

        Ok(())
    }

    fn advance_offsets(&mut self, tenant: &str, topic_name: &str, records: &[StorageRecord]) {
        for record in records {
            let key = (
                tenant.to_string(),
                topic_name.to_string(),
                record.metadata.shard.clone(),
            );
            let next = record.metadata.offset + 1;
            let entry = self.current_offsets.entry(key).or_insert(0);
            if next > *entry {
                *entry = next;
            }
        }
    }
}
