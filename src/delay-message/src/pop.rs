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

use std::{sync::Arc, time::Duration};

use crate::DelayMessageManager;
use common_base::error::common::CommonError;
use futures::StreamExt;
use metadata_struct::{
    adapter::{read_config::ReadConfig, record::Record},
    delay_info::DelayMessageInfo,
};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::{error, info};

pub async fn pop_delay_queue(
    namespace: &str,
    message_storage_adapter: &ArcStorageAdapter,
    delay_message_manager: &Arc<DelayMessageManager>,
    shard_no: u64,
) {
    if let Some(mut delay_queue) = delay_message_manager.delay_queue_list.get_mut(&shard_no) {
        while let Some(expired) = delay_queue.next().await {
            let delay_message = expired.into_inner();
            let raw_message_storage_adapter = message_storage_adapter.clone();
            let raw_namespace = namespace.to_owned();
            tokio::spawn(async move {
                send_delay_message_to_shard(
                    &raw_message_storage_adapter,
                    &raw_namespace,
                    delay_message,
                )
                .await;
            });
        }
    }
}

async fn send_delay_message_to_shard(
    message_storage_adapter: &ArcStorageAdapter,
    namespace: &str,
    delay_message: DelayMessageInfo,
) {
    let mut times = 0;
    info!(
        "send_delay_message_to_shard start,namespace:{},shard_name:{},offset:{}",
        namespace, delay_message.target_shard_name, delay_message.offset
    );

    loop {
        if times > 100 {
            error!("send_delay_message_to_shard failed, times: {},namespace:{},shard_name:{},offset:{}", times, namespace, delay_message.target_shard_name, delay_message.offset);
            break;
        }

        times += 1;
        let record = match read_offset_data(
            message_storage_adapter,
            namespace,
            &delay_message.delay_shard_name,
            delay_message.offset,
        )
        .await
        {
            Ok(Some(record)) => record,
            Ok(None) => break,
            Err(e) => {
                error!("read_offset_data failed, err: {:?}", e);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                continue;
            }
        };

        match message_storage_adapter
            .write(namespace, &delay_message.target_shard_name, &record)
            .await
        {
            Ok(id) => {
                info!("Delay message: message was written to {:?} successfully, offset: {:?}, delay info: {:?}",delay_message.target_shard_name,id, delay_message);
                break;
            }
            Err(e) => {
                error!("write failed, err: {:?}", e);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                continue;
            }
        }
    }
}

pub(crate) async fn read_offset_data(
    message_storage_adapter: &ArcStorageAdapter,
    namespace: &str,
    shard_name: &str,
    offset: u64,
) -> Result<Option<Record>, CommonError> {
    let read_config = ReadConfig {
        max_record_num: 1,
        max_size: 1024 * 1024 * 1024,
    };
    let results = message_storage_adapter
        .read_by_offset(namespace, shard_name, offset, &read_config)
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
    use std::{sync::Arc, time::Duration};

    use common_base::tools::unique_id;
    use metadata_struct::{adapter::record::Record, delay_info::DelayMessageInfo};
    use storage_adapter::storage::build_memory_storage_driver;
    use tokio::time::sleep;

    use crate::{
        pop::{read_offset_data, send_delay_message_to_shard},
        start_delay_message_pop, DelayMessageManager,
    };

    #[tokio::test]
    pub async fn read_offset_data_test() {
        let message_storage_adapter = build_memory_storage_driver();
        let namespace = unique_id();
        let shard_name = "s1".to_string();
        for i in 0..100 {
            let data = Record::build_str(format!("data{i}"));
            let res = message_storage_adapter
                .write(&namespace, &shard_name, &data)
                .await;
            assert!(res.is_ok());
        }

        for i in 0..100 {
            let res = read_offset_data(&message_storage_adapter, &namespace, &shard_name, i).await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.offset.unwrap(), i);

            let d: String = serde_json::from_slice(&raw.data).unwrap();
            assert_eq!(d, format!("data{i}"));
        }
    }

    #[tokio::test]
    pub async fn send_delay_message_to_shard_test() {
        let message_storage_adapter = build_memory_storage_driver();
        let namespace = unique_id();
        let shard_name = "s1".to_string();
        for i in 0..100 {
            let data = Record::build_str(format!("data{i}"));
            let res = message_storage_adapter
                .write(&namespace, &shard_name, &data)
                .await;
            assert!(res.is_ok());
        }

        let target_shard_name = unique_id();
        for i in 0..100 {
            let delay_message: DelayMessageInfo = DelayMessageInfo {
                delay_shard_name: shard_name.to_owned(),
                target_shard_name: target_shard_name.to_owned(),
                offset: i,
                delay_timestamp: 5,
            };
            send_delay_message_to_shard(&message_storage_adapter, &namespace, delay_message).await;
        }

        for i in 0..100 {
            let res =
                read_offset_data(&message_storage_adapter, &namespace, &target_shard_name, i).await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.offset.unwrap(), i);

            let d: String = serde_json::from_slice(&raw.data).unwrap();
            assert_eq!(d, format!("data{i}"));
        }
    }

    #[tokio::test]
    pub async fn pop_delay_queue_test() {
        let namespace = unique_id();
        let shard_num = 1;
        let message_storage_adapter = build_memory_storage_driver();
        let delay_message_manager = Arc::new(DelayMessageManager::new(
            namespace.clone(),
            shard_num,
            message_storage_adapter.clone(),
        ));
        delay_message_manager.start().await;

        start_delay_message_pop(
            &delay_message_manager,
            &message_storage_adapter,
            &namespace,
            shard_num,
        );

        let target_topic = unique_id();
        for i in 0..10 {
            let data = Record::build_str(format!("data{i}"));
            let res = delay_message_manager.send(&target_topic, i + 1, data).await;

            assert!(res.is_ok());
        }

        sleep(Duration::from_secs(15)).await;

        for i in 0..10 {
            let res =
                read_offset_data(&message_storage_adapter, &namespace, &target_topic, i).await;
            assert!(res.is_ok());
            let raw = res.unwrap().unwrap();
            assert_eq!(raw.offset.unwrap(), i);
            let d: String = serde_json::from_slice(&raw.data).unwrap();
            println!("i:{i},res:{d:?}")

            // assert_eq!(d, format!("data{}", i));
        }
    }
}
