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

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::server::common::connection_manager::ConnectionManager;
use crate::storage::message::MessageStorage;
use crate::subscribe::common::is_ignore_push_error;
use crate::subscribe::common::loop_commit_offset;
use crate::subscribe::common::Subscriber;
use crate::subscribe::manager::SubPushThreadData;
use crate::subscribe::manager::{ShareLeaderSubscribeData, SubscribeManager};
use crate::subscribe::push::{
    build_pub_qos, build_publish_message, build_sub_ids, send_publish_packet_to_client,
};
use common_base::network::broker_not_available;
use common_base::tools::now_second;
use metadata_struct::adapter::record::Record;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;
use tracing::debug;
use tracing::warn;
use tracing::{error, info};

#[derive(Clone)]
pub struct ShareLeaderPush<S> {
    pub subscribe_manager: Arc<SubscribeManager>,
    message_storage: Arc<S>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
}

impl<S> ShareLeaderPush<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        message_storage: Arc<S>,
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        ShareLeaderPush {
            subscribe_manager,
            message_storage,
            connection_manager,
            cache_manager,
        }
    }

    pub async fn start(&self) {
        loop {
            self.start_push_thread().await;
            self.try_thread_gc();
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub fn try_thread_gc(&self) {
        // Periodically verify that a push task is running, but the subscribe task has stopped
        // If so, stop the process and clean up the data
        for (share_leader_key, sx) in self.subscribe_manager.share_leader_push_thread.clone() {
            if !self
                .subscribe_manager
                .share_leader_push
                .contains_key(&share_leader_key)
            {
                match sx.sender.send(true) {
                    Ok(_) => {
                        self.subscribe_manager
                            .share_leader_push_thread
                            .remove(&share_leader_key);
                    }
                    Err(err) => {
                        error!("stop sub share thread error, error message:{}", err);
                    }
                }
            }
        }

        // gc
        for (key, raw) in self.subscribe_manager.share_leader_push.clone() {
            if raw.sub_list.is_empty() {
                self.subscribe_manager.share_leader_push.remove(&key);
            }
        }
    }

    pub async fn start_push_thread(&self) {
        // Periodically verify if any push tasks are not started. If so, the thread is started
        for (share_leader_key, sub_data) in self.subscribe_manager.share_leader_push.clone() {
            if sub_data.sub_list.is_empty() {
                if let Some(sx) = self
                    .subscribe_manager
                    .share_leader_push_thread
                    .get(&share_leader_key)
                {
                    if sx.sender.send(true).is_ok() {
                        self.subscribe_manager
                            .share_leader_push
                            .remove(&share_leader_key);
                    }
                }
            }

            // start push data thread
            if !self
                .subscribe_manager
                .share_leader_push_thread
                .contains_key(&share_leader_key)
            {
                if let Err(e) = self.push_by_round_robin(share_leader_key, sub_data).await {
                    error!("{:?}", e);
                }
            }
        }
    }

    async fn push_by_round_robin(
        &self,
        share_leader_key: String,
        sub_data: ShareLeaderSubscribeData,
    ) -> ResultMqttBrokerError {
        let (sub_thread_stop_sx, mut sub_thread_stop_rx) = broadcast::channel(1);
        let group_id = format!(
            "system_sub_{}_{}_{}",
            sub_data.group_name, sub_data.sub_name, sub_data.topic_id
        );

        // get current offset by group
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let mut offset = message_storage.get_group_offset(&group_id).await?;

        // save push thread
        self.subscribe_manager.share_leader_push_thread.insert(
            share_leader_key.clone(),
            SubPushThreadData {
                push_success_record_num: 0,
                push_error_record_num: 0,
                last_push_time: 0,
                last_run_time: 0,
                create_time: now_second(),
                sender: sub_thread_stop_sx.clone(),
            },
        );

        let connection_manager = self.connection_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let subscribe_manager = self.subscribe_manager.clone();

        tokio::spawn(async move {
            info!(
                "Share leader push data thread for GroupName {}/{},Topic [{}] was started successfully",
                sub_data.group_name, sub_data.sub_name, sub_data.topic_name
            );

            let mut seq = 1;
            loop {
                select! {
                    val = sub_thread_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                info!(
                                    "Share sub push data thread for GroupName {},Topic [{}] was stopped successfully",
                                    sub_data.group_name, sub_data.topic_name
                                );
                                break;
                            }
                        }
                    }
                    res = read_message_process(
                        &connection_manager,
                        &cache_manager,
                        &message_storage,
                        &subscribe_manager,
                        &share_leader_key,
                        &sub_data,
                        &group_id,
                        offset,
                        seq,
                        &sub_thread_stop_sx,
                    ) =>{
                        match res {
                            Ok((data,seq_num)) => {
                                if let Some(offset_cur) = data{
                                    offset = offset_cur + 1;
                                    seq = seq_num;
                                }else{
                                    sleep(Duration::from_millis(100)).await;
                                }
                            },

                            Err(e) => {
                                if !is_ignore_push_error(&e){
                                    error!(
                                        "Failed to read message from storage, failure message: {},topic:{},group{}",
                                        e.to_string(),
                                        &sub_data.topic_id,
                                        group_id
                                    );
                                    sleep(Duration::from_millis(100)).await;
                                }
                                break;
                            }
                        }
                    }
                }
            }

            subscribe_manager
                .share_leader_push_thread
                .remove(&share_leader_key);
        });
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
async fn read_message_process<S>(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
    message_storage: &MessageStorage<S>,
    subscribe_manager: &Arc<SubscribeManager>,
    share_leader_key: &str,
    sub_data: &ShareLeaderSubscribeData,
    group_id: &str,
    offset: u64,
    mut seq: u64,
    stop_sx: &Sender<bool>,
) -> Result<(Option<u64>, u64), MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let results = message_storage
        .read_topic_message(&sub_data.topic_id, offset, 100)
        .await?;

    let mut push_fn = async |record: &Record| -> ResultMqttBrokerError {
        let record_offset = if let Some(offset) = record.offset {
            offset
        } else {
            return Ok(());
        };

        let mut times = 0;
        loop {
            seq += 1;
            times += 1;
            if times > 3 {
                warn!("Shared subscription failed to send messages {} times and the messages were discarded,, offset: {:?}", times, record.offset);
                break;
            }

            let subscriber = if let Some(subscrbie) =
                get_subscribe_by_random(subscribe_manager, share_leader_key, seq)
            {
                subscrbie
            } else {
                warn!("No available subscribers were obtained. Continue looking for the next one, , offset: {:?}", record.offset);
                sleep(Duration::from_secs(1)).await;
                continue;
            };

            let qos = build_pub_qos(cache_manager, &subscriber);
            let sub_ids = build_sub_ids(&subscriber);

            // build publish params
            let sub_pub_param = match build_publish_message(
                cache_manager,
                connection_manager,
                &subscriber.client_id,
                record.to_owned(),
                group_id,
                &qos,
                &subscriber,
                &sub_ids,
            )
            .await
            {
                Ok(Some(param)) => param,
                Ok(None) => {
                    warn!(
                        "Build message is empty. group:{}, topic_id:{}, offset: {:?}",
                        group_id, sub_data.topic_id, record.offset
                    );
                    break;
                }
                Err(e) => {
                    warn!(
                        "Build message error. Error message : {}, offset: {:?}",
                        e, record.offset
                    );
                    break;
                }
            };

            if let Err(e) = send_publish_packet_to_client(
                connection_manager,
                cache_manager,
                &sub_pub_param,
                &qos,
                stop_sx,
            )
            .await
            {
                if broker_not_available(&e.to_string()) {
                    subscribe_manager.add_not_push_client(&subscriber.client_id);
                }

                debug!(
                    "Shared subscription failed to send a message to client {}. I attempted to
                    send it to the next client. Error message :{}, offset: {:?}",
                    subscriber.client_id, e, record.offset
                );
                continue;
            };

            break;
        }

        // commit offset
        loop_commit_offset(message_storage, &sub_data.topic_id, group_id, record_offset).await?;
        Ok(())
    };

    let mut success_num = 0;
    let mut error_num = 0;
    for record in results.iter() {
        match push_fn(record).await {
            Ok(_) => {
                success_num += 1;
            }
            Err(e) => {
                error_num += 1;
                if !is_ignore_push_error(&e) {
                    warn!(
                        "Share leader push fail, offset [{:?}], error message:{},",
                        record.offset, e
                    );
                }
            }
        }
    }

    subscribe_manager.update_subscribe_leader_push_thread_info(
        share_leader_key,
        success_num as u64,
        error_num as u64,
    );

    if results.is_empty() {
        return Ok((None, seq));
    }

    Ok((results.last().unwrap().offset, seq))
}

fn get_subscribe_by_random(
    subscribe_manager: &Arc<SubscribeManager>,
    share_leader_key: &str,
    mut seq: u64,
) -> Option<Subscriber> {
    loop {
        if let Some(sub_list) = subscribe_manager.share_leader_push.get(share_leader_key) {
            let index = seq % (sub_list.sub_list.len() as u64);
            let keys: Vec<String> = sub_list
                .sub_list
                .iter()
                .map(|entry| entry.key().clone())
                .collect();

            if let Some(key) = keys.get(index as usize) {
                if let Some(subscribe) = sub_list.sub_list.get(key) {
                    if !subscribe_manager
                        .not_push_client
                        .contains_key(&subscribe.client_id)
                    {
                        return Some(subscribe.clone());
                    }
                }
            }
        }
        seq += 1;
    }
}

#[cfg(test)]
mod tests {}
