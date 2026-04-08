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
    /// `base_tag` has the form `{tenant}_{mail_id}`; each consumer appends its own priority
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
