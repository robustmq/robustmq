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

pub mod delay;
pub mod driver;
pub mod handler;
pub mod manager;
pub mod pop;
pub mod recover;

use crate::delay::init_inner_topic;
use crate::manager::DelayTaskManager;
use crate::pop::spawn_delay_task_pop_threads;
use crate::recover::recover_delay_queue;
use broker_core::cache::BrokerCacheManager;
use bytes::Bytes;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_base::uuid::unique_id;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const DELAY_TASK_INDEX_TOPIC: &str = "$delay-task-index";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DelayTaskType {
    MQTTSessionExpire,
    MQTTLastwillExpire,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayTask {
    pub task_id: String,
    pub task_type: DelayTaskType,
    pub task_data: Bytes,
    pub delay_target_time: u64,
    pub create_time: u64,
    pub persistent: bool,
}

impl DelayTask {
    pub fn build_persistent(
        task_type: DelayTaskType,
        task_data: Bytes,
        delay_target_time: u64,
    ) -> Self {
        DelayTask {
            task_id: unique_id(),
            task_type,
            task_data,
            delay_target_time,
            create_time: now_second(),
            persistent: true,
        }
    }

    pub fn build_ephemeral(
        task_type: DelayTaskType,
        task_data: Bytes,
        delay_target_time: u64,
    ) -> Self {
        DelayTask {
            task_id: unique_id(),
            task_type,
            task_data,
            delay_target_time,
            create_time: now_second(),
            persistent: false,
        }
    }
}

pub async fn start_delay_task_manager_thread(
    delay_task_manager: &Arc<DelayTaskManager>,
    broker_cache: &Arc<BrokerCacheManager>,
) -> Result<(), CommonError> {
    delay_task_manager.start();

    init_inner_topic(delay_task_manager, broker_cache).await?;
    recover_delay_queue(delay_task_manager).await;
    spawn_delay_task_pop_threads(delay_task_manager, delay_task_manager.delay_queue_num);

    Ok(())
}
