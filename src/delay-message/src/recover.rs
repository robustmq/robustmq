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

use crate::delay::DELAY_QUEUE_INDEX_TOPIC;
use crate::manager::DelayMessageManager;
use crate::pop::delay_message_process;
use common_base::tools::now_second;
use common_base::utils::serialize::{self};
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

const MAX_READ_RETRY: u32 = 3;
const PROGRESS_LOG_INTERVAL: u64 = 1000;

pub(crate) async fn recover_delay_queue(delay_message_manager: &Arc<DelayMessageManager>) {
    info!("Starting delay queue recovery from persistent storage");

    let read_config = AdapterReadConfig {
        max_record_num: 100,
        max_size: 10 * 1024 * 1024,
    };

    let mut offsets = HashMap::new();
    let mut total_num = 0;
    let mut last_progress_log = 0;
    let mut retry_count = 0;

    loop {
        let data = match read_delay_index_batch(
            delay_message_manager,
            &offsets,
            &read_config,
            &mut retry_count,
        )
        .await
        {
            Some(data) => data,
            None => return,
        };

        if data.is_empty() {
            break;
        }

        for record in &data {
            if process_delay_index_record(delay_message_manager, record, &mut total_num).await
                && total_num - last_progress_log >= PROGRESS_LOG_INTERVAL
            {
                info!(
                    "Delay queue recovery progress: {} messages recovered",
                    total_num
                );
                last_progress_log = total_num;
            }
        }

        update_offsets_from_records(&data, &mut offsets);
    }

    info!(
        "Delay queue recovery completed. Total messages recovered: {}",
        total_num
    );
}

async fn read_delay_index_batch(
    delay_message_manager: &Arc<DelayMessageManager>,
    offsets: &HashMap<String, u64>,
    read_config: &AdapterReadConfig,
    retry_count: &mut u32,
) -> Option<Vec<metadata_struct::storage::storage_record::StorageRecord>> {
    match delay_message_manager
        .storage_driver_manager
        .read_by_offset(DELAY_QUEUE_INDEX_TOPIC, offsets, read_config)
        .await
    {
        Ok(data) => {
            *retry_count = 0;
            Some(data)
        }
        Err(e) => {
            *retry_count += 1;
            error!(
                "Reading shard {} failed (attempt {}/{}): {:?}",
                DELAY_QUEUE_INDEX_TOPIC, retry_count, MAX_READ_RETRY, e
            );

            if *retry_count >= MAX_READ_RETRY {
                error!(
                    "Failed to recover delay queue after {} retries, aborting recovery",
                    MAX_READ_RETRY
                );
                return None;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
            None
        }
    }
}

async fn process_delay_index_record(
    delay_message_manager: &Arc<DelayMessageManager>,
    record: &metadata_struct::storage::storage_record::StorageRecord,
    total_num: &mut u64,
) -> bool {
    let delay_info = match serialize::deserialize::<DelayMessageIndexInfo>(&record.data) {
        Ok(info) => info,
        Err(e) => {
            error!(
                "Failed to deserialize delay index at shard={}, offset={}: {:?}",
                DELAY_QUEUE_INDEX_TOPIC, record.metadata.offset, e
            );
            return false;
        }
    };

    let now = now_second();
    if delay_info.target_timestamp < now {
        handle_expired_delay_message(delay_message_manager, delay_info, now).await;
        return false;
    }

    delay_message_manager.send_to_delay_queue(&delay_info).await;
    *total_num += 1;
    true
}

async fn handle_expired_delay_message(
    delay_message_manager: &Arc<DelayMessageManager>,
    delay_info: DelayMessageIndexInfo,
    now: u64,
) {
    warn!(
        "Delay message expired during recovery, sending immediately. \
         offset: {}, target: {}, expired by: {}s",
        delay_info.offset,
        delay_info.target_topic_name,
        now - delay_info.target_timestamp
    );

    let manager = delay_message_manager.clone();
    tokio::spawn(async move {
        if let Err(e) =
            delay_message_process(&manager.storage_driver_manager, &delay_info, now_second()).await
        {
            error!(
                "Failed to send expired delay message (offset: {}): {:?}",
                delay_info.offset, e
            );
        }
    });
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
