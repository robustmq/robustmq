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

use crate::DelayTask;
use broker_core::inner_topic::DELAY_TASK_INDEX_TOPIC;
use common_base::error::common::CommonError;
use common_base::utils::serialize::serialize;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::tenant::DEFAULT_TENANT;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tracing::debug;

pub(crate) async fn save_delay_task_index(
    storage_driver_manager: &Arc<StorageDriverManager>,
    task: &DelayTask,
) -> Result<(), CommonError> {
    let data = serialize(task)?;
    let record =
        AdapterWriteRecord::new(DELAY_TASK_INDEX_TOPIC, data).with_key(task.task_id.clone());

    let result = storage_driver_manager
        .write(DEFAULT_TENANT, DELAY_TASK_INDEX_TOPIC, &[record])
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
        "Delay task index persisted: task_id={}, task_type={}",
        task.task_id,
        task.task_type_name()
    );
    Ok(())
}

pub(crate) async fn delete_delay_task_index(
    storage_driver_manager: &Arc<StorageDriverManager>,
    task_id: &str,
) -> Result<(), CommonError> {
    storage_driver_manager
        .delete_by_keys(DEFAULT_TENANT, DELAY_TASK_INDEX_TOPIC, &[task_id])
        .await?;
    debug!("Deleted delay task index: task_id={}", task_id);
    Ok(())
}
