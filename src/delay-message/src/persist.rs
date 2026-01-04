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

use common_base::{error::common::CommonError, tools::now_second, utils::serialize};
use metadata_struct::{
    delay_info::DelayMessageInfo, storage::adapter_read_config::AdapterReadConfig,
    storage::adapter_record::AdapterWriteRecord,
};
use std::{sync::Arc, time::Duration};
use storage_adapter::driver::ArcStorageAdapter;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::{pop::delay_message_process, DelayMessageManager};

pub const DELAY_QUEUE_INFO_SHARD_NAME: &str = "$delay-queue-info-shard";

pub async fn persist_delay_info(
    message_storage_adapter: &ArcStorageAdapter,
    delay_info: DelayMessageInfo,
) -> Result<(), CommonError> {
    let data = AdapterWriteRecord::from_bytes(serialize::serialize(&delay_info)?);
    message_storage_adapter
        .write(DELAY_QUEUE_INFO_SHARD_NAME, &data)
        .await?;
    Ok(())
}

pub async fn recover_delay_queue(
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

            let delay_info = match serialize::deserialize::<DelayMessageInfo>(&record.data) {
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

#[cfg(test)]
mod test {
    use crate::{
        delay::init_delay_message_shard,
        persist::{persist_delay_info, recover_delay_queue, DELAY_QUEUE_INFO_SHARD_NAME},
        pop::read_offset_data,
        start_delay_message_pop, DelayMessageManager,
    };
    use common_base::{tools::unique_id, utils::serialize};
    use common_config::storage::StorageAdapterType;
    use metadata_struct::{
        delay_info::DelayMessageInfo,
        storage::{
            adapter_offset::AdapterShardInfo, adapter_read_config::AdapterReadConfig,
            adapter_record::AdapterWriteRecord,
        },
    };
    use std::{sync::Arc, time::Duration};
    use storage_adapter::storage::{build_memory_storage_driver, build_storage_driver_manager};
    use tokio::time::sleep;

    #[tokio::test]
    pub async fn persist_delay_info_test() {
        let message_storage_adapter = build_memory_storage_driver();

        let target_shard_name = unique_id();
        let delay_shard_name = unique_id();
        message_storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: target_shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        message_storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: delay_shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        init_delay_message_shard(&message_storage_adapter, &StorageAdapterType::Memory, 10)
            .await
            .unwrap();
        for i in 0..10 {
            let delay_info = DelayMessageInfo {
                delay_shard_name: delay_shard_name.to_owned(),
                target_shard_name: target_shard_name.to_owned(),
                offset: i,
                delay_timestamp: 5,
                shard_no: 0,
            };

            let res = persist_delay_info(&message_storage_adapter, delay_info).await;
            assert!(res.is_ok());
        }

        for i in 0..10 {
            let res =
                read_offset_data(&message_storage_adapter, DELAY_QUEUE_INFO_SHARD_NAME, i).await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.pkid, i);

            let d = serialize::deserialize::<DelayMessageInfo>(&raw.data).unwrap();
            assert_eq!(d.target_shard_name, target_shard_name);
            assert_eq!(d.delay_shard_name, delay_shard_name);
            assert_eq!(d.offset, i);
            assert_eq!(d.delay_timestamp, 5);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    pub async fn build_delay_queue_test() {
        let shard_num = 1;
        let storage_driver_manager = build_storage_driver_manager().await.unwrap();
        let delay_message_manager = Arc::new(
            DelayMessageManager::new(
                storage_driver_manager.clone(),
                StorageAdapterType::Memory,
                shard_num,
            )
            .await
            .unwrap(),
        );
        delay_message_manager.start().await;

        let target_topic = unique_id();
        delay_message_manager
            .message_storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: target_topic.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        init_delay_message_shard(
            &delay_message_manager.message_storage_adapter,
            &StorageAdapterType::Memory,
            10,
        )
        .await
        .unwrap();
        for i in 0..10 {
            let data = AdapterWriteRecord::from_string(format!("data{i}"));
            // Use fixed delay to maintain order
            let res: Result<(), common_base::error::common::CommonError> =
                delay_message_manager.send(&target_topic, 2, data).await;

            assert!(res.is_ok());
        }

        let new_delay_message_manager = Arc::new(
            DelayMessageManager::new(
                storage_driver_manager,
                StorageAdapterType::Memory,
                shard_num,
            )
            .await
            .unwrap(),
        );
        new_delay_message_manager.start().await;

        start_delay_message_pop(
            &new_delay_message_manager,
            &new_delay_message_manager.message_storage_adapter,
            shard_num,
        );

        // build delay queue
        let read_config = AdapterReadConfig {
            max_record_num: 100,
            max_size: 1024 * 1024 * 1024,
        };

        recover_delay_queue(
            &new_delay_message_manager.message_storage_adapter,
            &new_delay_message_manager,
            read_config,
            shard_num,
        )
        .await;

        // Wait for all messages to expire (2 seconds + buffer)
        sleep(Duration::from_secs(3)).await;

        // Give spawned tasks time to complete
        sleep(Duration::from_millis(500)).await;

        // Collect all messages and verify content (order may vary for concurrent expiry)
        let mut received_data = std::collections::HashSet::new();
        for i in 0..10 {
            let res = read_offset_data(
                &new_delay_message_manager.message_storage_adapter,
                &target_topic,
                i,
            )
            .await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.pkid, i);

            let d = String::from_utf8(raw.data.to_vec()).unwrap();
            received_data.insert(d);
        }

        // Verify all expected messages were received
        for i in 0..10 {
            assert!(
                received_data.contains(&format!("data{i}")),
                "Missing data{i} in received messages"
            );
        }
    }
}
