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
use log::error;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use storage_adapter::storage::StorageAdapter;

pub async fn pop_delay_queue<S>(
    namespace: &str,
    message_storage_adapter: &Arc<S>,
    delay_message_manager: &Arc<DelayMessageManager<S>>,
    shard_no: u64,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    if let Some(mut delay_queue) = delay_message_manager.delay_queue_list.get_mut(&shard_no) {
        while let Some(expired) = delay_queue.next().await {
            let delay_message = expired.into_inner();
            let raw_message_storage_adapter = message_storage_adapter.clone();
            let raw_namespace = namespace.to_owned();
            tokio::spawn(async move {
                send_delay_message_to_shard(
                    &raw_message_storage_adapter,
                    &raw_namespace,
                    &delay_message.shard_name,
                    delay_message.offset,
                )
                .await;
            });
        }
    }
}

async fn send_delay_message_to_shard<S>(
    message_storage_adapter: &Arc<S>,
    namespace: &str,
    shard_name: &str,
    offset: u64,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    loop {
        let record =
            match read_offset_data(message_storage_adapter, namespace, shard_name, offset).await {
                Ok(Some(record)) => record,
                Ok(None) => break,
                Err(e) => {
                    error!("read_offset_data failed, err: {:?}", e);
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
            };

        match message_storage_adapter
            .write(namespace.to_owned(), shard_name.to_owned(), record)
            .await
        {
            Ok(_) => {
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

async fn read_offset_data<S>(
    message_storage_adapter: &Arc<S>,
    namespace: &str,
    shard_name: &str,
    offset: u64,
) -> Result<Option<Record>, CommonError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let read_config = ReadConfig {
        max_record_num: 1,
        max_size: 1024 * 1024 * 1024,
    };
    let results = message_storage_adapter
        .read_by_offset(
            namespace.to_owned(),
            shard_name.to_owned(),
            offset,
            read_config,
        )
        .await?;

    for record in results {
        if record.offset.unwrap() == offset {
            return Ok(Some(record));
        }
    }
    Ok(None)
}
