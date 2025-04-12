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

use common_base::tools::now_second;
use tracing::error;
use metadata_struct::adapter::read_config::ReadConfig;
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::time::sleep;

use crate::{DelayMessageManager, DelayMessageRecord};

pub async fn build_delay_queue<S>(
    message_storage_adapter: Arc<S>,
    delay_message_manager: Arc<DelayMessageManager<S>>,
    namespace: String,
    shard_no: u64,
    shard_name: String,
    read_config: ReadConfig,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let mut offset = 0;

    loop {
        let data = match message_storage_adapter
            .read_by_offset(
                namespace.to_owned(),
                shard_name.to_owned(),
                offset,
                read_config.clone(),
            )
            .await
        {
            Ok(data) => data,
            Err(e) => {
                error!("Reading the shard {} failed with error * while building the deferred message index {:?}", shard_name, e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        if data.is_empty() {
            break;
        }
        for record in data {
            offset = record.offset.unwrap();

            if record.delay_timestamp < now_second() {
                continue;
            }

            let delay_message_record = DelayMessageRecord {
                shard_name: shard_name.to_owned(),
                offset,
                delay_timestamp: record.delay_timestamp,
            };

            if let Err(e) = delay_message_manager
                .send_to_delay_queue(shard_no, delay_message_record)
                .await
            {
                error!("While building the deferred message index, sending a message to the DelayQueue failed with error message :{:?}", e);
            }
        }
    }
}
