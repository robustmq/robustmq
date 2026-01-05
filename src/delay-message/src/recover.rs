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

use crate::delay::DELAY_QUEUE_INFO_SHARD_NAME;
use crate::manager::DelayMessageManager;
use crate::pop::delay_message_process;
use common_base::tools::now_second;
use common_base::utils::serialize::{self};
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::ArcStorageAdapter;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

pub(crate) fn start_recover_delay_queue(
    delay_message_manager: &Arc<DelayMessageManager>,
    message_storage_adapter: &ArcStorageAdapter,
    shard_num: u64,
) {
    let read_config = AdapterReadConfig {
        max_record_num: 100,
        max_size: 1024 * 1024 * 1024,
    };

    info!(
        "Starting delay queue recovery from persistent storage (shards: {})",
        shard_num
    );

    let new_delay_message_manager = delay_message_manager.clone();
    let new_message_storage_adapter = message_storage_adapter.clone();
    tokio::spawn(async move {
        recover_delay_queue(
            &new_message_storage_adapter,
            &new_delay_message_manager,
            read_config,
            shard_num,
        )
        .await;
    });
}

pub(crate) async fn recover_delay_queue(
    message_storage_adapter: &ArcStorageAdapter,
    delay_message_manager: &Arc<DelayMessageManager>,
    read_config: AdapterReadConfig,
    _shard_num: u64,
) {
    let mut offset = 0;
    let mut total_num = 0;
    loop {
        let data = match message_storage_adapter
            .read_by_offset(DELAY_QUEUE_INFO_SHARD_NAME, offset, &read_config)
            .await
        {
            Ok(data) => data,
            Err(e) => {
                error!("Reading the shard {} failed with error * while building the deferred message index {:?}", DELAY_QUEUE_INFO_SHARD_NAME, e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        if data.is_empty() {
            break;
        }

        for record in data {
            offset = record.metadata.offset;

            let delay_info = match serialize::deserialize::<DelayMessageIndexInfo>(&record.data) {
                Ok(delay_info) => delay_info,
                Err(e) => {
                    error!("While building the deferred message index, parsing the message failed with error message :{:?}", e);
                    continue;
                }
            };

            let now = now_second();
            if delay_info.delay_timestamp < now {
                warn!(
                    "Delay message expired during recovery, sending immediately. \
                     Delay shard: {}, offset: {}, target: {}, expired by: {}s",
                    delay_info.delay_shard_name,
                    delay_info.offset,
                    delay_info.target_shard_name,
                    now - delay_info.delay_timestamp
                );

                let storage = message_storage_adapter.clone();
                let info = delay_info.clone();
                tokio::spawn(async move {
                    if let Err(e) = delay_message_process(&storage, &info).await {
                        debug!(
                            "Failed to send expired delay message (shard: {}, offset: {}): {:?}",
                            info.delay_shard_name, info.offset, e
                        );
                    }
                });
                continue;
            }

            delay_message_manager.send_to_delay_queue(delay_info.shard_no, &delay_info);

            total_num += 1;
        }

        offset += 1;
    }
    info!("Delay queue index was successfully constructed from the persistent store. Number of data items: {}", total_num);
}
