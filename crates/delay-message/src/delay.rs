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

use common_base::error::common::CommonError;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use storage_adapter::storage::{ArcStorageAdapter, ShardInfo};
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{debug, info};

use crate::{persist::recover_delay_queue, pop::pop_delay_queue, DelayMessageManager};

const DELAY_MESSAGE_SHARD_NAME_PREFIX: &str = "$delay-message-shard-";

pub(crate) fn start_recover_delay_queue(
    delay_message_manager: &Arc<DelayMessageManager>,
    message_storage_adapter: &ArcStorageAdapter,
    namespace: &str,
    shard_num: u64,
) {
    let read_config = ReadConfig {
        max_record_num: 100,
        max_size: 1024 * 1024 * 1024,
    };

    let new_delay_message_manager = delay_message_manager.clone();
    let new_message_storage_adapter = message_storage_adapter.clone();
    let new_namespace = namespace.to_owned();
    tokio::spawn(async move {
        recover_delay_queue(
            &new_message_storage_adapter,
            &new_delay_message_manager,
            &new_namespace,
            read_config,
            shard_num,
        )
        .await;
    });
}

pub(crate) fn start_delay_message_pop(
    delay_message_manager: &Arc<DelayMessageManager>,
    message_storage_adapter: &ArcStorageAdapter,
    namespace: &str,
    shard_num: u64,
) {
    for shard_no in 0..shard_num {
        let new_delay_message_manager = delay_message_manager.clone();
        let new_message_storage_adapter = message_storage_adapter.clone();
        let new_namespace = namespace.to_owned();

        let (stop_send, _) = broadcast::channel(2);
        delay_message_manager.add_delay_queue_pop_thread(shard_no, stop_send.clone());

        tokio::spawn(async move {
            loop {
                let mut recv = stop_send.subscribe();
                select! {
                    val = recv.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                debug!("{}","Heartbeat reporting thread exited successfully");
                                break;
                            }
                        }
                    }
                    _ =  pop_delay_queue(
                        &new_namespace,
                        &new_message_storage_adapter,
                        &new_delay_message_manager,
                        shard_no,
                    ) => {
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });
    }
}

pub(crate) async fn persist_delay_message(
    message_storage_adapter: &ArcStorageAdapter,
    namespace: &str,
    shard_name: &str,
    data: Record,
) -> Result<u64, CommonError> {
    let offset = message_storage_adapter
        .write(namespace.to_owned(), shard_name.to_owned(), data.clone())
        .await?;

    Ok(offset)
}

pub(crate) async fn init_delay_message_shard(
    message_storage_adapter: &ArcStorageAdapter,
    namespace: &str,
    shard_num: u64,
) -> Result<(), CommonError> {
    for i in 0..shard_num {
        let shard_name = get_delay_message_shard_name(i);
        let results = message_storage_adapter
            .list_shard(namespace.to_owned(), shard_name.clone())
            .await?;

        if results.is_empty() {
            let shard = ShardInfo {
                namespace: namespace.to_owned(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            };
            message_storage_adapter.create_shard(shard).await?;
            info!("init shard:{}, {}", namespace, shard_name);
        }
    }

    Ok(())
}

pub(crate) fn get_delay_message_shard_name(no: u64) -> String {
    format!("{DELAY_MESSAGE_SHARD_NAME_PREFIX}{no}")
}

#[cfg(test)]
mod test {
    use common_base::tools::unique_id;
    use metadata_struct::adapter::record::Record;
    use storage_adapter::storage::build_memory_storage_driver;

    use crate::{
        get_delay_message_shard_name, init_delay_message_shard, persist_delay_message,
        pop::read_offset_data,
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
        let message_storage_adapter = build_memory_storage_driver();
        let namespace = unique_id();
        let shard_num = 1;
        let res = init_delay_message_shard(&message_storage_adapter, &namespace, shard_num).await;
        assert!(res.is_ok());

        let shard_name = get_delay_message_shard_name(shard_num - 1);
        let res = message_storage_adapter
            .list_shard(namespace, shard_name.clone())
            .await;
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res.first().unwrap().shard_name, shard_name);
    }

    #[tokio::test]
    pub async fn persist_delay_message_test() {
        let message_storage_adapter = build_memory_storage_driver();
        let namespace = unique_id();
        let shard_name = "test".to_string();
        let data = Record::build_str("test".to_string());
        let res =
            persist_delay_message(&message_storage_adapter, &namespace, &shard_name, data).await;
        assert!(res.is_ok());
        let offset = res.unwrap();

        let res = read_offset_data(&message_storage_adapter, &namespace, &shard_name, offset).await;
        assert!(res.is_ok());
        let res = res.unwrap().unwrap();
        assert_eq!(res.offset.unwrap(), offset);
        let d1: String = serde_json::from_slice(&res.data).unwrap();
        assert_eq!(d1, "test".to_string());
    }
}
