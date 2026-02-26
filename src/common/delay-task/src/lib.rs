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
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_base::uuid::unique_id;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const DELAY_TASK_INDEX_TOPIC: &str = "$delay-task-index";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DelayTaskData {
    MQTTSessionExpire(String),
    MQTTLastwillExpire(String),
}

impl DelayTaskData {
    pub fn task_type_name(&self) -> &'static str {
        match self {
            DelayTaskData::MQTTSessionExpire(_) => "MQTTSessionExpire",
            DelayTaskData::MQTTLastwillExpire(_) => "MQTTLastwillExpire",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayTask {
    pub task_id: String,
    pub data: DelayTaskData,
    pub delay_target_time: u64,
    pub create_time: u64,
    pub persistent: bool,
}

impl DelayTask {
    pub fn build_persistent(task_id: String, data: DelayTaskData, delay_target_time: u64) -> Self {
        DelayTask {
            task_id,
            data,
            delay_target_time,
            create_time: now_second(),
            persistent: true,
        }
    }

    pub fn build_persistent_auto_id(data: DelayTaskData, delay_target_time: u64) -> Self {
        Self::build_persistent(unique_id(), data, delay_target_time)
    }

    pub fn build_ephemeral(task_id: String, data: DelayTaskData, delay_target_time: u64) -> Self {
        DelayTask {
            task_id,
            data,
            delay_target_time,
            create_time: now_second(),
            persistent: false,
        }
    }

    pub fn build_ephemeral_auto_id(data: DelayTaskData, delay_target_time: u64) -> Self {
        Self::build_ephemeral(unique_id(), data, delay_target_time)
    }

    pub fn task_type_name(&self) -> &'static str {
        self.data.task_type_name()
    }
}

pub async fn start_delay_task_manager_thread(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    delay_task_manager: &Arc<DelayTaskManager>,
    broker_cache: &Arc<BrokerCacheManager>,
) -> Result<(), CommonError> {
    delay_task_manager.start();

    init_inner_topic(delay_task_manager, broker_cache).await?;
    spawn_delay_task_pop_threads(
        rocksdb_engine_handler,
        delay_task_manager,
        delay_task_manager.delay_queue_num,
    );

    let recover_manager = delay_task_manager.clone();
    let recover_engine = rocksdb_engine_handler.clone();
    tokio::spawn(async move {
        recover_delay_queue(&recover_engine, &recover_manager).await;
    });

    Ok(())
}
