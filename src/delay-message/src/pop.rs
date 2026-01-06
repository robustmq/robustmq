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

use crate::delay::{delete_delay_index_info, delete_delay_message};
use crate::driver::get_storage_driver;
use crate::manager::DelayMessageManager;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use futures::StreamExt;
use grpc_clients::meta::storage::call::list_shard;
use metadata_struct::storage::shard::EngineShard;
use metadata_struct::{
    delay_info::DelayMessageIndexInfo, storage::adapter_read_config::AdapterReadConfig,
    storage::adapter_record::AdapterWriteRecord,
    storage::convert::convert_engine_record_to_adapter,
};
use protocol::meta::meta_service_journal::ListShardRequest;
use std::sync::Arc;
use storage_adapter::driver::ArcStorageAdapter;
use tokio::{select, sync::broadcast};
use tracing::{debug, error, info, warn};

pub(crate) fn spawn_delay_message_pop_threads(
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
                        match val {
                            Ok(flag) if flag => {
                                info!("Delay message pop thread stopped for shard {}", shard_no);
                                break;
                            }
                            Err(_) => {
                                warn!("Broadcast channel closed, stopping pop thread for shard {}", shard_no);
                                break;
                            }
                            _ => {}
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
pub async fn pop_delay_queue(
    message_storage_adapter: &ArcStorageAdapter,
    delay_message_manager: &Arc<DelayMessageManager>,
    shard_no: u64,
) {
    if let Some(mut delay_queue) = delay_message_manager.delay_queue_list.get_mut(&shard_no) {
        if let Some(expired) = delay_queue.next().await {
            let delay_message = expired.into_inner();
            // Drop the lock before spawning to avoid holding it
            drop(delay_queue);

            // Spawn task to send delay message to avoid blocking the pop loop
            let storage = message_storage_adapter.clone();
            let raw_delay_message_manager = delay_message_manager.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    delay_message_process(&raw_delay_message_manager, &storage, &delay_message)
                        .await
                {
                    error!(
                        "Failed to process delay message: shard={}, offset={}, target={}, error={}",
                        delay_message.delay_shard_name,
                        delay_message.offset,
                        delay_message.target_shard_name,
                        e
                    );
                }
            });
        }
    }
}

pub async fn delay_message_process(
    delay_message_manager: &Arc<DelayMessageManager>,
    message_storage_adapter: &ArcStorageAdapter,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    let offset =
        send_delay_message_to_shard(delay_message_manager, message_storage_adapter, delay_info)
            .await?;
    delete_delay_index_info(message_storage_adapter, delay_info).await?;
    delete_delay_message(message_storage_adapter, delay_info).await?;
    debug!(
        "Delay message processed successfully, target offset: {}",
        offset
    );
    Ok(())
}

async fn send_delay_message_to_shard(
    delay_message_manager: &Arc<DelayMessageManager>,
    message_storage_adapter: &ArcStorageAdapter,
    delay_message: &DelayMessageIndexInfo,
) -> Result<u64, CommonError> {
    // read data
    let Some(record) = read_offset_data(
        message_storage_adapter,
        &delay_message.delay_shard_name,
        delay_message.offset,
    )
    .await?
    else {
        return Err(CommonError::CommonError(format!(
            "Delay message not found: shard={}, offset={}",
            delay_message.delay_shard_name, delay_message.offset
        )));
    };

    // send to target topic
    let target_shard_engine_type = if let Some(engine_type) =
        get_shard_storage_type(delay_message_manager, delay_message).await?
    {
        engine_type
    } else {
        return Err(CommonError::CommonError("".to_string()));
    };

    let target_message_storage_adapter = get_storage_driver(
        &delay_message_manager.storage_driver_manager,
        &target_shard_engine_type,
    )?;

    let resp = target_message_storage_adapter
        .write(&delay_message.target_shard_name, &record)
        .await?;
    if resp.is_error() {
        return Err(CommonError::CommonError(resp.error_info()));
    }
    debug!(
        "Expired delay message sent successfully: {} -> {} (offset: {})",
        delay_message.delay_shard_name, delay_message.target_shard_name, delay_message.offset
    );
    Ok(resp.offset)
}

pub(crate) async fn read_offset_data(
    message_storage_adapter: &ArcStorageAdapter,
    shard_name: &str,
    offset: u64,
) -> Result<Option<AdapterWriteRecord>, CommonError> {
    let read_config = AdapterReadConfig {
        max_record_num: 1,
        max_size: 10 * 1024 * 1024,
    };
    let results = message_storage_adapter
        .read_by_offset(shard_name, offset, &read_config)
        .await?;

    Ok(results
        .into_iter()
        .find(|r| r.metadata.offset == offset)
        .map(convert_engine_record_to_adapter))
}

#[cfg(test)]
mod test {
    use crate::{
        delay::{save_delay_index_info, save_delay_message},
        pop::{delay_message_process, read_offset_data, send_delay_message_to_shard},
    };
    use common_base::tools::unique_id;
    use metadata_struct::{
        delay_info::DelayMessageIndexInfo,
        storage::{adapter_offset::AdapterShardInfo, adapter_record::AdapterWriteRecord},
    };
    use std::sync::Arc;
    use storage_adapter::storage::test_build_memory_storage_driver;

    #[tokio::test]
    async fn send_delay_message_test() {
        let adapter = test_build_memory_storage_driver();
        let delay_shard = unique_id();
        let target_shard = unique_id();
        let manager = Arc::new(crate::manager::DelayMessageManager::new_for_test(
            adapter.clone(),
            1,
        ));

        adapter
            .create_shard(&AdapterShardInfo {
                shard_name: delay_shard.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        adapter
            .create_shard(&AdapterShardInfo {
                shard_name: target_shard.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let data = AdapterWriteRecord::from_string("test_data".to_string());
        let delay_offset = save_delay_message(&adapter, &delay_shard, data)
            .await
            .unwrap();

        let delay_info = DelayMessageIndexInfo {
            unique_id: unique_id(),
            delay_shard_name: delay_shard.clone(),
            target_shard_name: target_shard.clone(),
            offset: delay_offset,
            delay_timestamp: 0,
            shard_no: 0,
        };

        let target_offset = send_delay_message_to_shard(&manager, &adapter, &delay_info)
            .await
            .unwrap();

        let record = read_offset_data(&adapter, &target_shard, target_offset)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(record.data.to_vec()).unwrap(),
            "test_data"
        );
    }

    #[tokio::test]
    async fn delay_message_process_test() {
        let adapter = test_build_memory_storage_driver();
        let delay_shard = unique_id();
        let target_shard = unique_id();
        let manager = Arc::new(crate::manager::DelayMessageManager::new_for_test(
            adapter.clone(),
            1,
        ));

        adapter
            .create_shard(&AdapterShardInfo {
                shard_name: delay_shard.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        adapter
            .create_shard(&AdapterShardInfo {
                shard_name: target_shard.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        adapter
            .create_shard(&AdapterShardInfo {
                shard_name: crate::delay::DELAY_QUEUE_INFO_SHARD_NAME.to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        let data = AdapterWriteRecord::from_string("test_data".to_string());
        let delay_offset = save_delay_message(&adapter, &delay_shard, data)
            .await
            .unwrap();

        let delay_info = DelayMessageIndexInfo {
            unique_id: unique_id(),
            delay_shard_name: delay_shard.clone(),
            target_shard_name: target_shard.clone(),
            offset: delay_offset,
            delay_timestamp: 0,
            shard_no: 0,
        };

        save_delay_index_info(&adapter, &delay_info).await.unwrap();

        delay_message_process(&manager, &adapter, &delay_info)
            .await
            .unwrap();

        let target_record = read_offset_data(&adapter, &target_shard, 0)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(target_record.data.to_vec()).unwrap(),
            "test_data"
        );

        let delay_record = read_offset_data(&adapter, &delay_shard, delay_offset).await;
        assert!(delay_record.is_ok());
        assert!(delay_record.unwrap().is_none());
    }
}
