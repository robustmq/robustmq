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
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

const MAX_READ_RETRY: u32 = 3;
const PROGRESS_LOG_INTERVAL: u64 = 1000;

pub(crate) async fn recover_delay_queue(delay_message_manager: &Arc<DelayMessageManager>) {
    info!("Starting delay queue recovery from persistent storage",);

    let mut total_num = 0;
    let mut retry_count = 0;
    let mut last_progress_log = 0;
    let read_config = AdapterReadConfig {
        max_record_num: 100,
        max_size: 10 * 1024 * 1024,
    };

    let offsets = HashMap::new();
    loop {
        let data = match delay_message_manager
            .storage_driver_manager
            .read_by_offset(DELAY_QUEUE_INDEX_TOPIC, &offsets, &read_config)
            .await
        {
            Ok(data) => {
                retry_count = 0;
                data
            }
            Err(e) => {
                retry_count += 1;
                error!(
                    "Reading shard {}  failed (attempt {}/{}): {:?}",
                    DELAY_QUEUE_INDEX_TOPIC, retry_count, MAX_READ_RETRY, e
                );

                if retry_count >= MAX_READ_RETRY {
                    error!(
                        "Failed to recover delay queue after {} retries, aborting recovery",
                        MAX_READ_RETRY
                    );
                    return;
                }

                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        if data.is_empty() {
            break;
        }

        for record in &data {
            let delay_info = match serialize::deserialize::<DelayMessageIndexInfo>(&record.data) {
                Ok(delay_info) => delay_info,
                Err(e) => {
                    error!(
                        "Failed to deserialize delay message index at shard={}, offset={}: {:?}",
                        DELAY_QUEUE_INDEX_TOPIC, record.metadata.offset, e
                    );
                    continue;
                }
            };

            let now = now_second();
            if delay_info.delay_timestamp < now {
                warn!(
                    "Delay message expired during recovery, sending immediately. \
                     Delay offset: {}, target: {}, expired by: {}s",
                    delay_info.offset,
                    delay_info.target_topic_name,
                    now - delay_info.delay_timestamp
                );

                let info = delay_info.clone();
                let manager = delay_message_manager.clone();
                tokio::spawn(async move {
                    if let Err(e) =
                        delay_message_process(&manager.storage_driver_manager, &info).await
                    {
                        debug!(
                            "Failed to send expired delay message (offset: {}): {:?}",
                            info.offset, e
                        );
                    }
                });
                continue;
            }

            delay_message_manager.send_to_delay_queue(&delay_info);
            total_num += 1;

            if total_num - last_progress_log >= PROGRESS_LOG_INTERVAL {
                info!(
                    "Delay queue recovery progress: {} messages recovered so far",
                    total_num
                );
                last_progress_log = total_num;
            }
        }
    }

    info!(
        "Delay queue index was successfully constructed from the persistent store. Total messages recovered: {}",
        total_num
    );
}
