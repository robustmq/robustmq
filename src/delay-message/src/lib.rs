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

use build::build_delay_queue;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use std::{
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use storage_adapter::storage::{ShardInfo, StorageAdapter};
use tokio::{
    select,
    sync::broadcast,
    time::{sleep, Instant},
};
use tokio_util::time::DelayQueue;
use tracing::{debug, info};

pub mod build;
pub mod pop;

#[derive(Clone)]
pub struct DelayMessageRecord {
    shard_name: String,
    offset: u64,
    delay_timestamp: u64,
}

pub struct DelayMessageManager<S> {
    namespace: String,
    shard_num: u64,
    message_storage_adapter: Arc<S>,
    incr_no: AtomicU64,
    delay_queue_list: DashMap<u64, DelayQueue<DelayMessageRecord>>,
    delay_queue_pop_thread: DashMap<u64, broadcast::Sender<bool>>,
}

const DELAY_MESSAGE_SHARD_NAME_PREFIX: &str = "$delay-message-shard-";
impl<S> DelayMessageManager<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(namespace: String, shard_num: u64, message_storage_adapter: Arc<S>) -> Self {
        DelayMessageManager {
            namespace,
            shard_num,
            message_storage_adapter,
            incr_no: AtomicU64::new(0),
            delay_queue_list: DashMap::with_capacity(2),
            delay_queue_pop_thread: DashMap::with_capacity(2),
        }
    }

    pub async fn init(&self) -> Result<(), CommonError> {
        self.try_init_shard().await?;
        self.init_delay_queue().await?;
        info!("DelayMessage service start");
        Ok(())
    }

    async fn try_init_shard(&self) -> Result<(), CommonError> {
        for i in 0..self.shard_num {
            let shard_name = self.get_delay_message_shard_name(i);
            let results = self
                .message_storage_adapter
                .list_shard(self.namespace.clone(), shard_name.clone())
                .await?;
            if results.is_empty() {
                let shard = ShardInfo {
                    namespace: self.namespace.clone(),
                    shard_name: shard_name.clone(),
                    replica_num: 1,
                };
                self.message_storage_adapter.create_shard(shard).await?;
                info!("init shard:{}, {}", self.namespace.clone(), shard_name);
            }
        }

        Ok(())
    }

    async fn init_delay_queue(&self) -> Result<(), CommonError> {
        for shard_no in 0..self.shard_num {
            let delay_queue = DelayQueue::new();
            self.delay_queue_list.insert(shard_no, delay_queue);
        }
        Ok(())
    }

    pub async fn send_delay_message(&self, data: Record) -> Result<(), CommonError> {
        let shard_no = self.get_target_shard_no();
        let namespace = self.namespace.clone();
        let shard_name = self.get_delay_message_shard_name(shard_no);
        let offset = self
            .message_storage_adapter
            .write(namespace, shard_name.clone(), data.clone())
            .await?;

        let delay_message_record = DelayMessageRecord {
            shard_name,
            offset,
            delay_timestamp: data.delay_timestamp,
        };
        self.send_to_delay_queue(shard_no, delay_message_record)
            .await?;
        Ok(())
    }

    fn add_delay_queue_pop_thread(&self, shard_no: u64, stop_send: broadcast::Sender<bool>) {
        self.delay_queue_pop_thread.insert(shard_no, stop_send);
    }

    pub async fn stop(&self) -> Result<(), CommonError> {
        for shard_no in 0..self.shard_num {
            if let Some(stop_send) = self.delay_queue_pop_thread.get(&shard_no) {
                stop_send.send(true)?;
            }
        }
        Ok(())
    }

    async fn send_to_delay_queue(
        &self,
        shard_no: u64,
        delay_message_record: DelayMessageRecord,
    ) -> Result<(), CommonError> {
        if let Some(mut delay_queue) = self.delay_queue_list.get_mut(&shard_no) {
            delay_queue.insert_at(
                delay_message_record.clone(),
                Instant::now() + Duration::from_secs(delay_message_record.delay_timestamp),
            );
        }
        Ok(())
    }

    fn get_target_shard_no(&self) -> u64 {
        self.incr_no
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.shard_num
    }

    fn get_delay_message_shard_name(&self, no: u64) -> String {
        format!("{}{}", DELAY_MESSAGE_SHARD_NAME_PREFIX, no)
    }
}

pub async fn start_build_delay_queue<S>(
    namespace: String,
    delay_message_manager: Arc<DelayMessageManager<S>>,
    message_storage_adapter: Arc<S>,
    shard_num: u64,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    for shard_no in 0..shard_num {
        let shard_name = delay_message_manager.get_delay_message_shard_name(shard_no);
        let read_config = ReadConfig {
            max_record_num: 100,
            max_size: 1024 * 1024 * 1024,
        };

        let new_delay_message_manager = delay_message_manager.clone();
        let new_message_storage_adapter = message_storage_adapter.clone();
        let new_namespace = namespace.clone();
        tokio::spawn(async move {
            build_delay_queue(
                new_message_storage_adapter,
                new_delay_message_manager,
                new_namespace,
                shard_no,
                shard_name,
                read_config,
            )
            .await;
        });
    }
}

pub async fn start_delay_message_pop<S>(
    namespace: String,
    message_storage_adapter: Arc<S>,
    delay_message_manager: Arc<DelayMessageManager<S>>,
    shard_num: u64,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
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
                    _ =  pop::pop_delay_queue(
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
