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

use crate::driver::build_delay_message_shard_config;
use crate::manager::DelayMessageManager;
use crate::pop::pop_delay_queue;
use bytes::Bytes;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_base::utils::serialize::serialize;
use common_config::storage::StorageAdapterType;
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::{adapter_offset::AdapterShardInfo, shard::EngineShardConfig};
use std::sync::Arc;
use storage_adapter::driver::ArcStorageAdapter;
use tokio::{select, sync::broadcast};
use tracing::{debug, info};

const DELAY_MESSAGE_SHARD_NAME_PREFIX: &str = "$delay-message-shard-";
pub const DELAY_QUEUE_INFO_SHARD_NAME: &str = "$delay_queue_index_info_shard";

pub(crate) fn start_delay_message_pop(
    delay_message_manager: &Arc<DelayMessageManager>,
    message_storage_adapter: &ArcStorageAdapter,
    shard_num: u64,
) {
    info!("Starting delay message pop threads (shards: {})", shard_num);

    for shard_no in 0..shard_num {
        let new_delay_message_manager = delay_message_manager.clone();
        let new_message_storage_adapter = message_storage_adapter.clone();

        let (stop_send, _) = broadcast::channel(2);
        delay_message_manager.add_delay_queue_pop_thread(shard_no, stop_send.clone());

        tokio::spawn(async move {
            info!("Delay message pop thread started for shard {}", shard_no);
            let mut recv = stop_send.subscribe();
            loop {
                select! {
                    val = recv.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                info!("Delay message pop thread stopped for shard {}", shard_no);
                                break;
                            }
                        }
                    }
                    _ =  pop_delay_queue(
                        &new_message_storage_adapter,
                        &new_delay_message_manager,
                        shard_no,
                    ) => {
                        // Yield to other tasks to avoid tight loops when many messages expire
                        tokio::task::yield_now().await;
                    }
                }
            }
        });
    }
}

pub(crate) async fn save_delay_message(
    message_storage_adapter: &ArcStorageAdapter,
    shard_name: &str,
    data: AdapterWriteRecord,
) -> Result<u64, CommonError> {
    let resp = message_storage_adapter.write(shard_name, &data).await?;
    if resp.is_error() {
        return Err(CommonError::CommonError(resp.error_info()));
    }

    debug!(
        "Delay message persisted to shard {} at offset {}",
        shard_name, resp.offset
    );

    Ok(resp.offset)
}

pub(crate) async fn delete_delay_message(
    message_storage_adapter: &ArcStorageAdapter,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    message_storage_adapter
        .delete_by_offset(&delay_info.delay_shard_name, delay_info.offset)
        .await
}

pub async fn save_delay_index_info(
    message_storage_adapter: &ArcStorageAdapter,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    let data = serialize(&delay_info)?;
    let data = AdapterWriteRecord {
        key: Some(delay_info.unique_id.clone()),
        data: Bytes::copy_from_slice(&data),
        timestamp: now_second(),
        ..Default::default()
    };
    message_storage_adapter
        .write(DELAY_QUEUE_INFO_SHARD_NAME, &data)
        .await?;
    Ok(())
}

pub(crate) async fn delete_delay_index_info(
    message_storage_adapter: &ArcStorageAdapter,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    message_storage_adapter
        .delete_by_key(DELAY_QUEUE_INFO_SHARD_NAME, &delay_info.unique_id)
        .await?;
    Ok(())
}

pub(crate) async fn init_delay_message_shard(
    message_storage_adapter: &ArcStorageAdapter,
    engine_storage_type: &StorageAdapterType,
    shard_num: u64,
) -> Result<(), CommonError> {
    let mut created_count = 0;
    for i in 0..shard_num {
        let shard_name = get_delay_message_shard_name(i);
        let results = message_storage_adapter
            .list_shard(Some(shard_name.clone()))
            .await?;
        if results.is_empty() {
            let shard = AdapterShardInfo {
                shard_name: shard_name.clone(),
                config: build_delay_message_shard_config(engine_storage_type)?,
            };
            message_storage_adapter.create_shard(&shard).await?;
            debug!("Created delay message shard: {}", shard_name);
            created_count += 1;
        }
    }

    let results = message_storage_adapter
        .list_shard(Some(DELAY_QUEUE_INFO_SHARD_NAME.to_string()))
        .await?;
    if results.is_empty() {
        let shard = AdapterShardInfo {
            shard_name: DELAY_QUEUE_INFO_SHARD_NAME.to_string(),
            config: EngineShardConfig::default(),
        };
        message_storage_adapter.create_shard(&shard).await?;
        debug!(
            "Created delay message shard: {}",
            DELAY_QUEUE_INFO_SHARD_NAME
        );
        created_count += 1;
    }

    info!(
        "Delay message shards initialized: {} total, {} newly created",
        shard_num, created_count
    );

    Ok(())
}

pub(crate) fn get_delay_message_shard_name(no: u64) -> String {
    format!("{DELAY_MESSAGE_SHARD_NAME_PREFIX}{no}")
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, time::Duration};

    use common_base::{tools::unique_id, utils::serialize};
    use common_config::storage::StorageAdapterType;
    use metadata_struct::{
        delay_info::DelayMessageIndexInfo,
        storage::{
            adapter_offset::AdapterShardInfo, adapter_read_config::AdapterReadConfig,
            adapter_record::AdapterWriteRecord,
        },
    };
    use storage_adapter::storage::{
        test_build_memory_storage_driver, test_build_storage_driver_manager,
    };
    use tokio::time::sleep;

    use crate::{
        delay::{
            get_delay_message_shard_name, init_delay_message_shard, save_delay_index_info,
            save_delay_message, start_delay_message_pop, DELAY_QUEUE_INFO_SHARD_NAME,
        },
        manager::DelayMessageManager,
        pop::read_offset_data,
        recover::recover_delay_queue,
    };

    #[tokio::test]
    pub async fn get_delay_message_shard_name_test() {
        assert_eq!(
            get_delay_message_shard_name(0),
            "$delay-message-shard-0".to_string()
        );

        assert_eq!(
            get_delay_message_shard_name(1),
            "$delay-message-shard-1".to_string()
        );

        assert_eq!(
            get_delay_message_shard_name(2),
            "$delay-message-shard-2".to_string()
        );
    }

    #[tokio::test]
    pub async fn init_delay_message_shard_test() {
        let message_storage_adapter = test_build_memory_storage_driver();
        let shard_num = 1;
        let res = init_delay_message_shard(
            &message_storage_adapter,
            &StorageAdapterType::Memory,
            shard_num,
        )
        .await;
        assert!(res.is_ok());

        let shard_name = get_delay_message_shard_name(shard_num - 1);
        let res = message_storage_adapter
            .list_shard(Some(shard_name.clone()))
            .await;
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res.first().unwrap().shard_name, shard_name);
    }

    #[tokio::test]
    pub async fn persist_delay_message_test() {
        let message_storage_adapter = test_build_memory_storage_driver();
        let shard_name = "test".to_string();
        let data = AdapterWriteRecord::from_string("test".to_string());
        message_storage_adapter
            .create_shard(&AdapterShardInfo {
                shard_name: shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        let res = save_delay_message(&message_storage_adapter, &shard_name, data).await;
        assert!(res.is_ok());
        let offset = res.unwrap();

        let res = read_offset_data(&message_storage_adapter, &shard_name, offset).await;
        assert!(res.is_ok());
        let res: AdapterWriteRecord = res.unwrap().unwrap();
        assert_eq!(res.pkid, offset);
        let d1 = String::from_utf8(res.data.to_vec()).unwrap();
        assert_eq!(d1, "test".to_string());
    }
    #[tokio::test]
    pub async fn persist_delay_info_test() {
        let message_storage_adapter = test_build_memory_storage_driver();

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
            let delay_info = DelayMessageIndexInfo {
                unique_id: unique_id(),
                delay_shard_name: delay_shard_name.to_owned(),
                target_shard_name: target_shard_name.to_owned(),
                offset: i,
                delay_timestamp: 5,
                shard_no: 0,
            };

            let res = save_delay_index_info(&message_storage_adapter, &delay_info).await;
            assert!(res.is_ok());
        }

        for i in 0..10 {
            let res =
                read_offset_data(&message_storage_adapter, DELAY_QUEUE_INFO_SHARD_NAME, i).await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.pkid, i);

            let d = serialize::deserialize::<DelayMessageIndexInfo>(&raw.data).unwrap();
            assert_eq!(d.target_shard_name, target_shard_name);
            assert_eq!(d.delay_shard_name, delay_shard_name);
            assert_eq!(d.offset, i);
            assert_eq!(d.delay_timestamp, 5);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    pub async fn build_delay_queue_test() {
        let shard_num = 1;
        let storage_driver_manager = test_build_storage_driver_manager().await.unwrap();
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
