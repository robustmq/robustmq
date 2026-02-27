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

use crate::manager::DelayTaskManager;
use crate::pop::spawn_task_process;
use crate::{DelayTask, DELAY_TASK_INDEX_TOPIC};
use broker_core::cache::BrokerCacheManager;
use common_base::tools::now_second;
use common_base::utils::serialize;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

const MAX_READ_RETRY: u32 = 3;
const PROGRESS_LOG_INTERVAL: u64 = 1000;

enum ReadBatch {
    Data(Vec<metadata_struct::storage::storage_record::StorageRecord>),
    Retry,
    Abort,
}

pub(crate) async fn recover_delay_queue(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    delay_task_manager: &Arc<DelayTaskManager>,
    node_call_manager: &Arc<NodeCallManager>,
    broker_cache: &Arc<BrokerCacheManager>,
) {
    info!("Starting delay task queue recovery from persistent storage");

    let read_config = AdapterReadConfig {
        max_record_num: 100,
        max_size: 10 * 1024 * 1024,
    };

    let mut offsets = HashMap::new();
    let mut recovered = 0u64;
    let mut expired = 0u64;
    let mut last_progress_log = 0u64;
    let mut retry_count = 0u32;

    loop {
        let data = match read_delay_task_batch(
            delay_task_manager,
            &offsets,
            &read_config,
            &mut retry_count,
        )
        .await
        {
            ReadBatch::Data(data) => data,
            ReadBatch::Retry => continue,
            ReadBatch::Abort => return,
        };

        if data.is_empty() {
            break;
        }

        for record in &data {
            match process_delay_task_record(
                rocksdb_engine_handler,
                delay_task_manager,
                node_call_manager,
                broker_cache,
                record,
            )
            .await
            {
                RecoverResult::Recovered => {
                    recovered += 1;
                    if recovered - last_progress_log >= PROGRESS_LOG_INTERVAL {
                        info!(
                            "Delay task queue recovery progress: {} recovered, {} expired",
                            recovered, expired
                        );
                        last_progress_log = recovered;
                    }
                }
                RecoverResult::Expired => {
                    expired += 1;
                }
                RecoverResult::Failed => {}
            }
        }

        update_offsets_from_records(&data, &mut offsets);
    }

    info!(
        "Delay task queue recovery completed. recovered: {}, expired: {}",
        recovered, expired
    );
}

async fn read_delay_task_batch(
    delay_task_manager: &Arc<DelayTaskManager>,
    offsets: &HashMap<String, u64>,
    read_config: &AdapterReadConfig,
    retry_count: &mut u32,
) -> ReadBatch {
    match delay_task_manager
        .storage_driver_manager
        .read_by_offset(DELAY_TASK_INDEX_TOPIC, offsets, read_config)
        .await
    {
        Ok(data) => {
            *retry_count = 0;
            ReadBatch::Data(data)
        }
        Err(e) => {
            *retry_count += 1;
            error!(
                "Reading topic {} failed (attempt {}/{}): {:?}",
                DELAY_TASK_INDEX_TOPIC, retry_count, MAX_READ_RETRY, e
            );

            if *retry_count >= MAX_READ_RETRY {
                error!(
                    "Failed to recover delay task queue after {} retries, aborting recovery",
                    MAX_READ_RETRY
                );
                return ReadBatch::Abort;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
            ReadBatch::Retry
        }
    }
}

enum RecoverResult {
    Recovered,
    Expired,
    Failed,
}

async fn process_delay_task_record(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    delay_task_manager: &Arc<DelayTaskManager>,
    node_call_manager: &Arc<NodeCallManager>,
    broker_cache: &Arc<BrokerCacheManager>,
    record: &metadata_struct::storage::storage_record::StorageRecord,
) -> RecoverResult {
    let task = match serialize::deserialize::<DelayTask>(&record.data) {
        Ok(t) => t,
        Err(e) => {
            error!(
                "Failed to deserialize delay task at topic={}, offset={}: {:?}",
                DELAY_TASK_INDEX_TOPIC, record.metadata.offset, e
            );
            return RecoverResult::Failed;
        }
    };

    let now = now_second();
    if task.delay_target_time < now {
        handle_expired_delay_task(
            rocksdb_engine_handler,
            delay_task_manager,
            node_call_manager,
            broker_cache,
            task,
            now,
        )
        .await;
        return RecoverResult::Expired;
    }

    delay_task_manager.enqueue_task(&task).await;
    RecoverResult::Recovered
}

async fn handle_expired_delay_task(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    delay_task_manager: &Arc<DelayTaskManager>,
    node_call_manager: &Arc<NodeCallManager>,
    broker_cache: &Arc<BrokerCacheManager>,
    task: DelayTask,
    now: u64,
) {
    warn!(
        "Delay task expired during recovery, executing immediately. \
         task_id={}, task_type={}, expired by: {}s",
        task.task_id,
        task.task_type_name(),
        now - task.delay_target_time
    );

    let manager = delay_task_manager.clone();
    spawn_task_process(
        rocksdb_engine_handler.clone(),
        manager,
        node_call_manager.clone(),
        broker_cache.clone(),
        task,
    )
    .await;
}

fn update_offsets_from_records(
    data: &[metadata_struct::storage::storage_record::StorageRecord],
    offsets: &mut HashMap<String, u64>,
) {
    for record in data {
        let current_offset = offsets.get(&record.metadata.shard).copied().unwrap_or(0);
        offsets.insert(
            record.metadata.shard.clone(),
            current_offset.max(record.metadata.offset + 1),
        );
    }
}
