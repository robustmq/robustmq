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

use crate::DelayMessageManager;
use common_base::error::common::CommonError;
use futures::StreamExt;
use metadata_struct::{
    adapter::{read_config::ReadConfig, record::Record},
    delay_info::DelayMessageInfo,
};
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;

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
            tokio::spawn(async move {
                if let Err(e) = send_delay_message_to_shard(&storage, delay_message).await {
                    error!("{}", e);
                }
            });
        }
    }
}

async fn send_delay_message_to_shard(
    message_storage_adapter: &ArcStorageAdapter,
    delay_message: DelayMessageInfo,
) -> Result<u64, CommonError> {
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

    message_storage_adapter
        .write(&delay_message.target_shard_name, &record)
        .await
}

pub(crate) async fn read_offset_data(
    message_storage_adapter: &ArcStorageAdapter,
    shard_name: &str,
    offset: u64,
) -> Result<Option<Record>, CommonError> {
    let read_config = ReadConfig {
        max_record_num: 1,
        max_size: 1024 * 1024 * 1024,
    };
    let results = message_storage_adapter
        .read_by_offset(shard_name, offset, &read_config)
        .await?;

    for record in results {
        if record.offset.unwrap() == offset {
            return Ok(Some(record));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod test {
    use crate::{
        delay::init_delay_message_shard,
        pop::{read_offset_data, send_delay_message_to_shard},
        start_delay_message_pop, DelayMessageManager,
    };
    use common_base::tools::unique_id;
    use metadata_struct::{
        adapter::{record::Record, ShardInfo},
        delay_info::DelayMessageInfo,
    };
    use std::{sync::Arc, time::Duration};
    use storage_adapter::storage::build_memory_storage_driver;
    use tokio::time::sleep;

    #[tokio::test]
    pub async fn read_offset_data_test() {
        let message_storage_adapter = build_memory_storage_driver();
        let shard_name = "s1".to_string();
        message_storage_adapter
            .create_shard(&ShardInfo {
                shard_name: shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        for i in 0..100 {
            let data = Record::from_string(format!("data{i}"));
            let res = message_storage_adapter.write(&shard_name, &data).await;
            assert!(res.is_ok());
        }

        for i in 0..100 {
            let res = read_offset_data(&message_storage_adapter, &shard_name, i).await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.offset.unwrap(), i);

            let d = String::from_utf8(raw.data.to_vec()).unwrap();
            assert_eq!(d, format!("data{i}"));
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    pub async fn send_delay_message_to_shard_test() {
        let message_storage_adapter = build_memory_storage_driver();
        let shard_name = "s1".to_string();
        message_storage_adapter
            .create_shard(&ShardInfo {
                shard_name: shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        for i in 0..100 {
            let data = Record::from_string(format!("data{i}"));
            let res = message_storage_adapter.write(&shard_name, &data).await;
            assert!(res.is_ok());
        }

        let target_shard_name = unique_id();
        message_storage_adapter
            .create_shard(&ShardInfo {
                shard_name: target_shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        for i in 0..100 {
            let delay_message: DelayMessageInfo = DelayMessageInfo {
                delay_shard_name: shard_name.to_owned(),
                target_shard_name: target_shard_name.to_owned(),
                offset: i,
                delay_timestamp: 5,
                shard_no: 0,
            };
            if let Err(e) =
                send_delay_message_to_shard(&message_storage_adapter, delay_message).await
            {
                println!(" {:?}", e);
            }
        }

        for i in 0..100 {
            let res = read_offset_data(&message_storage_adapter, &target_shard_name, i).await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.offset.unwrap(), i);

            let d = String::from_utf8(raw.data.to_vec()).unwrap();
            assert_eq!(d, format!("data{i}"));
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    pub async fn pop_delay_queue_test() {
        let shard_num = 1;
        let message_storage_adapter = build_memory_storage_driver();
        let delay_message_manager = Arc::new(DelayMessageManager::new(
            shard_num,
            message_storage_adapter.clone(),
        ));
        delay_message_manager.start().await;

        init_delay_message_shard(&message_storage_adapter, shard_num)
            .await
            .unwrap();
        start_delay_message_pop(&delay_message_manager, &message_storage_adapter, shard_num);

        let target_topic = unique_id();
        message_storage_adapter
            .create_shard(&ShardInfo {
                shard_name: target_topic.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        for i in 0..10 {
            let data = Record::from_string(format!("data{i}"));
            let res = delay_message_manager.send(&target_topic, 2, data).await;
            assert!(res.is_ok());
        }

        sleep(Duration::from_secs(3)).await;

        sleep(Duration::from_millis(500)).await;
        let mut received_data = std::collections::HashSet::new();
        for i in 0..10 {
            let res = read_offset_data(&message_storage_adapter, &target_topic, i).await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.offset.unwrap(), i);
            let d = String::from_utf8(raw.data.to_vec()).unwrap();
            received_data.insert(d);
        }

        for i in 0..10 {
            assert!(
                received_data.contains(&format!("data{i}")),
                "Missing data{i} in received messages"
            );
        }
    }
}
