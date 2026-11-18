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

use std::collections::HashMap;
use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::{
    mq9::Priority,
    storage::{adapter_read_config::AdapterReadConfig, record::StorageRecord},
};

use crate::{
    consumer::{GroupConsumer, StartOffsetStrategy},
    driver::StorageDriverManager,
    priority::storage_priority_tag,
};

pub struct PriorityGroupConsumer {
    critical_consumer: GroupConsumer,
    urgent_consumer: GroupConsumer,
    normal_consumer: GroupConsumer,
}

impl PriorityGroupConsumer {
    pub fn new(driver: Arc<StorageDriverManager>, group_name: String) -> Self {
        PriorityGroupConsumer {
            critical_consumer: GroupConsumer::new(
                driver.clone(),
                PriorityGroupConsumer::group_name(&group_name, &Priority::Critical),
            ),

            urgent_consumer: GroupConsumer::new(
                driver.clone(),
                PriorityGroupConsumer::group_name(&group_name, &Priority::Urgent),
            ),

            normal_consumer: GroupConsumer::new(
                driver.clone(),
                PriorityGroupConsumer::group_name(&group_name, &Priority::Normal),
            ),
        }
    }

    pub fn new_manual(driver: Arc<StorageDriverManager>, group_name: String) -> Self {
        PriorityGroupConsumer {
            critical_consumer: GroupConsumer::new_manual(
                driver.clone(),
                PriorityGroupConsumer::group_name(&group_name, &Priority::Critical),
            ),
            urgent_consumer: GroupConsumer::new_manual(
                driver.clone(),
                PriorityGroupConsumer::group_name(&group_name, &Priority::Urgent),
            ),
            normal_consumer: GroupConsumer::new_manual(
                driver.clone(),
                PriorityGroupConsumer::group_name(&group_name, &Priority::Normal),
            ),
        }
    }

    pub async fn set_start_offset_strategy(&self, strategy: StartOffsetStrategy) {
        self.critical_consumer
            .set_start_offset_strategy(strategy.clone())
            .await;
        self.urgent_consumer
            .set_start_offset_strategy(strategy.clone())
            .await;
        self.normal_consumer
            .set_start_offset_strategy(strategy.clone())
            .await;
    }

    pub async fn next_messages(
        &self,
        tenant: &str,
        topic_name: &str,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        // consumer critical
        let mut result = self
            .critical_consumer
            .next_messages(tenant, topic_name, read_config)
            .await?;

        if result.len() >= read_config.max_record_num as usize {
            return Ok(result);
        }

        // consumer urgent
        let urgent_data = self
            .urgent_consumer
            .next_messages(tenant, topic_name, read_config)
            .await?;
        result.extend_from_slice(&urgent_data);

        if result.len() >= read_config.max_record_num as usize {
            return Ok(result);
        }

        // consumer normal
        let normal_data = self
            .normal_consumer
            .next_messages(tenant, topic_name, read_config)
            .await?;
        result.extend_from_slice(&normal_data);

        if result.len() >= read_config.max_record_num as usize {
            return Ok(result);
        }

        Ok(result)
    }

    /// Read messages in priority order (critical → urgent → normal) using per-priority tags.
    /// `base_tag` has the form `{tenant}_{mail_address}`; each consumer appends its own priority
    /// suffix (`_critical`, `_urgent`, `_normal`) so that only matching records are returned.
    pub async fn next_messages_by_tags(
        &self,
        tenant: &str,
        topic_name: &str,
        tag: &str,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let critical_tag = storage_priority_tag(tag, &Priority::Critical);
        let mut result = self
            .critical_consumer
            .next_messages_by_tags(tenant, topic_name, &critical_tag, read_config)
            .await?;

        if result.len() >= read_config.max_record_num as usize {
            return Ok(result);
        }

        let urgent_tag = storage_priority_tag(tag, &Priority::Urgent);
        let urgent_data = self
            .urgent_consumer
            .next_messages_by_tags(tenant, topic_name, &urgent_tag, read_config)
            .await?;
        result.extend_from_slice(&urgent_data);

        if result.len() >= read_config.max_record_num as usize {
            return Ok(result);
        }

        let normal_tag = storage_priority_tag(tag, &Priority::Normal);
        let normal_data = self
            .normal_consumer
            .next_messages_by_tags(tenant, topic_name, &normal_tag, read_config)
            .await?;
        result.extend_from_slice(&normal_data);

        Ok(result)
    }

    /// Stage the same shard offsets into all three priority consumers so a single
    /// `commit()` call persists them for every priority group.
    pub fn stage_offsets(&self, tenant: &str, topic: &str, shard_offsets: &HashMap<String, u64>) {
        for (shard, &offset) in shard_offsets {
            self.critical_consumer
                .stage_shard_offset(tenant, topic, shard, offset);
            self.urgent_consumer
                .stage_shard_offset(tenant, topic, shard, offset);
            self.normal_consumer
                .stage_shard_offset(tenant, topic, shard, offset);
        }
    }

    pub fn set_current_offsets(
        &self,
        tenant: &str,
        topic: &str,
        shard_offsets: &HashMap<String, u64>,
    ) {
        self.critical_consumer
            .set_current_offsets(tenant, topic, shard_offsets);
        self.urgent_consumer
            .set_current_offsets(tenant, topic, shard_offsets);
        self.normal_consumer
            .set_current_offsets(tenant, topic, shard_offsets);
    }

    pub async fn commit(&self) -> Result<(), CommonError> {
        self.critical_consumer.commit().await?;
        self.urgent_consumer.commit().await?;
        self.normal_consumer.commit().await?;
        Ok(())
    }

    pub fn advance(&self) {
        self.critical_consumer.advance();
        self.urgent_consumer.advance();
        self.normal_consumer.advance();
    }

    fn group_name(group: &str, priority: &Priority) -> String {
        format!("{}-{}", group, priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        priority::storage_priority_tag,
        storage::{test_add_topic, test_build_storage_driver_manager},
    };
    use common_base::uuid::unique_id;
    use metadata_struct::{adapter::adapter_record::AdapterWriteRecord, tenant::DEFAULT_TENANT};

    fn read_config(max_record_num: u64) -> AdapterReadConfig {
        AdapterReadConfig {
            max_record_num,
            max_size: 1024 * 1024,
        }
    }

    async fn seed_group_cache(manager: &Arc<StorageDriverManager>, group: &str) {
        for priority in [Priority::Critical, Priority::Urgent, Priority::Normal] {
            manager
                .commit_group_offset(DEFAULT_TENANT, &format!("{group}-{priority}"), &[])
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn priority_tag_reads_are_ordered_and_limited() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let topic_name = unique_id();
        test_add_topic(&manager, &topic_name);
        let base_tag = "mailbox";
        manager
            .write(
                DEFAULT_TENANT,
                &topic_name,
                &[
                    AdapterWriteRecord::new(&topic_name, b"normal".as_ref())
                        .with_tags(vec![storage_priority_tag(base_tag, &Priority::Normal)]),
                    AdapterWriteRecord::new(&topic_name, b"critical".as_ref())
                        .with_tags(vec![storage_priority_tag(base_tag, &Priority::Critical)]),
                    AdapterWriteRecord::new(&topic_name, b"urgent".as_ref())
                        .with_tags(vec![storage_priority_tag(base_tag, &Priority::Urgent)]),
                ],
                1,
            )
            .await
            .unwrap();

        let group_name = unique_id();
        seed_group_cache(&manager, &group_name).await;
        let consumer = PriorityGroupConsumer::new(manager, group_name);
        let records = consumer
            .next_messages_by_tags(DEFAULT_TENANT, &topic_name, base_tag, &read_config(2))
            .await
            .unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].data.as_ref(), b"critical");
        assert_eq!(records[1].data.as_ref(), b"urgent");
    }

    #[tokio::test]
    async fn stage_and_commit_apply_offsets_to_all_priority_consumers() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let group_name = unique_id();
        seed_group_cache(&manager, &group_name).await;
        let consumer = PriorityGroupConsumer::new_manual(manager.clone(), group_name.clone());
        let offsets = HashMap::from([(String::from("shard"), 7)]);

        consumer.stage_offsets(DEFAULT_TENANT, "topic", &offsets);
        consumer.commit().await.unwrap();

        for priority in [Priority::Critical, Priority::Urgent, Priority::Normal] {
            let group = format!("{group_name}-{priority}");
            let committed = manager
                .get_offset_by_group(DEFAULT_TENANT, &group)
                .await
                .unwrap();
            assert_eq!(committed.len(), 1);
            assert_eq!(committed[0].shard_name, "shard");
            assert_eq!(committed[0].offset, 8);
        }
    }
}
