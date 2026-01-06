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

use crate::delay::{delete_delay_index_info, delete_delay_message, DELAY_QUEUE_MESSAGE_TOPIC};
use crate::manager::DelayMessageManager;
use common_base::error::common::CommonError;
use futures::StreamExt;
use metadata_struct::{
    delay_info::DelayMessageIndexInfo, storage::convert::convert_engine_record_to_adapter,
};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::{select, sync::broadcast};
use tracing::{debug, error, info, warn};

pub(crate) fn spawn_delay_message_pop_threads(
    delay_message_manager: &Arc<DelayMessageManager>,
    delay_queue_num: u32,
) {
    info!("Starting delay message pop threads ( {})", delay_queue_num);

    for shard_no in 0..delay_queue_num {
        let new_delay_message_manager = delay_message_manager.clone();

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
pub async fn pop_delay_queue(delay_message_manager: &Arc<DelayMessageManager>, shard_no: u32) {
    if let Some(mut delay_queue) = delay_message_manager.delay_queue_list.get_mut(&shard_no) {
        if let Some(expired) = delay_queue.next().await {
            let delay_message = expired.into_inner();
            // Drop the lock before spawning to avoid holding it
            drop(delay_queue);

            // Spawn task to send delay message to avoid blocking the pop loop
            let raw_delay_message_manager = delay_message_manager.clone();
            tokio::spawn(async move {
                if let Err(e) = delay_message_process(
                    &raw_delay_message_manager.storage_driver_manager,
                    &delay_message,
                )
                .await
                {
                    error!(
                        "Failed to process delay message: offset={}, target={}, error={}",
                        delay_message.offset, delay_message.target_topic_name, e
                    );
                }
            });
        }
    }
}

pub async fn delay_message_process(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_info: &DelayMessageIndexInfo,
) -> Result<(), CommonError> {
    let offset = send_delay_message_to_shard(storage_driver_manager, delay_info).await?;
    delete_delay_index_info(storage_driver_manager, delay_info).await?;
    delete_delay_message(storage_driver_manager, &delay_info.unique_id).await?;
    debug!(
        "Delay message processed successfully, target offset: {}",
        offset
    );
    Ok(())
}

async fn send_delay_message_to_shard(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_message: &DelayMessageIndexInfo,
) -> Result<u64, CommonError> {
    // read data

    let results = storage_driver_manager
        .read_by_key(DELAY_QUEUE_MESSAGE_TOPIC, &delay_message.unique_id)
        .await?;

    if results.is_empty() {
        return Err(CommonError::CommonError("".to_string()));
    }

    if results.len() > 1 {
        return Err(CommonError::CommonError("".to_string()));
    }

    let record = if let Some(record) = results.first() {
        record.clone()
    } else {
        return Err(CommonError::CommonError("".to_string()));
    };

    let send_record = convert_engine_record_to_adapter(record);

    // send to target topic
    let resp = storage_driver_manager
        .write(&delay_message.target_topic_name, &[send_record])
        .await?;

    let write_resp = if let Some(data) = resp.first() {
        data.clone()
    } else {
        return Err(CommonError::CommonError("".to_string()));
    };

    if write_resp.is_error() {
        return Err(CommonError::CommonError(write_resp.error_info()));
    }
    debug!(
        "Expired delay message sent successfully: delay queue -> {} (offset: {})",
        delay_message.target_topic_name, delay_message.offset
    );
    Ok(write_resp.offset)
}

#[cfg(test)]
mod test {
    // use crate::{
    //     delay::{save_delay_index_info, save_delay_message},
    //     pop::{delay_message_process, send_delay_message_to_shard},
    // };
    // use common_base::tools::unique_id;
    // use metadata_struct::{
    //     delay_info::DelayMessageIndexInfo,
    //     storage::{adapter_offset::AdapterShardInfo, adapter_record::AdapterWriteRecord},
    // };
    // use std::sync::Arc;

    // #[tokio::test]
    // async fn send_delay_message_test() {
    //     let adapter = test_build_memory_storage_driver();
    //     let delay_shard = unique_id();
    //     let target_shard = unique_id();
    //     let manager = Arc::new(crate::manager::DelayMessageManager::new_for_test(
    //         adapter.clone(),
    //         1,
    //     ));

    //     adapter
    //         .create_shard(&AdapterShardInfo {
    //             shard_name: delay_shard.clone(),
    //             ..Default::default()
    //         })
    //         .await
    //         .unwrap();
    //     adapter
    //         .create_shard(&AdapterShardInfo {
    //             shard_name: target_shard.clone(),
    //             ..Default::default()
    //         })
    //         .await
    //         .unwrap();

    //     let data = AdapterWriteRecord::from_string("test_data".to_string());
    //     let delay_offset = save_delay_message(&adapter, &delay_shard, data)
    //         .await
    //         .unwrap();

    //     let delay_info = DelayMessageIndexInfo {
    //         unique_id: unique_id(),
    //         target_topic_name: target_shard.clone(),
    //         offset: delay_offset,
    //         delay_timestamp: 0,
    //         shard_no: 0,
    //     };

    //     let target_offset = send_delay_message_to_shard(&manager, &adapter, &delay_info)
    //         .await
    //         .unwrap();

    //     let record = read_offset_data(&adapter, &target_shard, target_offset)
    //         .await
    //         .unwrap()
    //         .unwrap();
    //     assert_eq!(
    //         String::from_utf8(record.data.to_vec()).unwrap(),
    //         "test_data"
    //     );
    // }

    // #[tokio::test]
    // async fn delay_message_process_test() {
    //     let adapter = test_build_memory_storage_driver();
    //     let delay_shard = unique_id();
    //     let target_shard = unique_id();
    //     let manager = Arc::new(crate::manager::DelayMessageManager::new_for_test(
    //         adapter.clone(),
    //         1,
    //     ));

    //     adapter
    //         .create_shard(&AdapterShardInfo {
    //             shard_name: delay_shard.clone(),
    //             ..Default::default()
    //         })
    //         .await
    //         .unwrap();
    //     adapter
    //         .create_shard(&AdapterShardInfo {
    //             shard_name: target_shard.clone(),
    //             ..Default::default()
    //         })
    //         .await
    //         .unwrap();
    //     adapter
    //         .create_shard(&AdapterShardInfo {
    //             shard_name: crate::delay::DELAY_QUEUE_INFO_SHARD_NAME.to_string(),
    //             ..Default::default()
    //         })
    //         .await
    //         .unwrap();

    //     let data = AdapterWriteRecord::from_string("test_data".to_string());
    //     let delay_offset = save_delay_message(&adapter, &delay_shard, data)
    //         .await
    //         .unwrap();

    //     let delay_info = DelayMessageIndexInfo {
    //         unique_id: unique_id(),
    //         delay_shard_name: delay_shard.clone(),
    //         target_topic_name: target_shard.clone(),
    //         offset: delay_offset,
    //         delay_timestamp: 0,
    //         shard_no: 0,
    //     };

    //     save_delay_index_info(&adapter, &delay_info).await.unwrap();

    //     delay_message_process(&manager, &adapter, &delay_info)
    //         .await
    //         .unwrap();

    //     let target_record = read_offset_data(&adapter, &target_shard, 0)
    //         .await
    //         .unwrap()
    //         .unwrap();
    //     assert_eq!(
    //         String::from_utf8(target_record.data.to_vec()).unwrap(),
    //         "test_data"
    //     );

    //     let delay_record = read_offset_data(&adapter, &delay_shard, delay_offset).await;
    //     assert!(delay_record.is_ok());
    //     assert!(delay_record.unwrap().is_none());
    // }
}
