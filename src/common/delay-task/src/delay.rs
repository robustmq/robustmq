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

use crate::driver::build_delay_task_shard_config;
use crate::manager::DelayTaskManager;
use crate::{DelayTask, DELAY_TASK_INDEX_TOPIC};
use broker_core::cache::BrokerCacheManager;
use bytes::Bytes;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_base::utils::serialize::serialize;
use common_base::uuid::unique_id;
use common_config::storage::StorageType;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use storage_adapter::topic::create_topic_full;
use tracing::debug;

pub(crate) async fn save_delay_task_index(
    storage_driver_manager: &Arc<StorageDriverManager>,
    task: &DelayTask,
) -> Result<(), CommonError> {
    let data = serialize(task)?;
    let record = AdapterWriteRecord {
        key: Some(task.task_id.clone()),
        data: Bytes::copy_from_slice(&data),
        timestamp: now_second(),
        ..Default::default()
    };
    let result = storage_driver_manager
        .write(DELAY_TASK_INDEX_TOPIC, &[record])
        .await?;

    let resp = result.first().ok_or_else(|| {
        CommonError::CommonError(format!(
            "Write response is empty when saving delay task index (task_id={}) to topic '{}'",
            task.task_id, DELAY_TASK_INDEX_TOPIC
        ))
    })?;

    if resp.is_error() {
        return Err(CommonError::CommonError(resp.error_info()));
    }

    debug!(
        "Delay task index persisted: task_id={}, task_type={:?}",
        task.task_id, task.task_type
    );
    Ok(())
}

pub(crate) async fn delete_delay_task_index(
    storage_driver_manager: &Arc<StorageDriverManager>,
    task_id: &str,
) -> Result<(), CommonError> {
    storage_driver_manager
        .delete_by_key(DELAY_TASK_INDEX_TOPIC, task_id)
        .await?;
    debug!("Deleted delay task index: task_id={}", task_id);
    Ok(())
}

pub(crate) async fn init_inner_topic(
    delay_task_manager: &Arc<DelayTaskManager>,
    broker_cache: &Arc<BrokerCacheManager>,
) -> Result<(), CommonError> {
    let uid = unique_id();
    let topic = Topic {
        topic_id: uid.clone(),
        topic_name: DELAY_TASK_INDEX_TOPIC.to_string(),
        storage_type: StorageType::EngineRocksDB,
        partition: 1,
        replication: 1,
        storage_name_list: Topic::create_partition_name(&uid, 1),
        create_time: now_second(),
    };
    let shard_config = build_delay_task_shard_config(&StorageType::EngineRocksDB)?;
    create_topic_full(
        broker_cache,
        &delay_task_manager.storage_driver_manager,
        &delay_task_manager.client_pool,
        &topic,
        &shard_config,
    )
    .await?;
    Ok(())
}
