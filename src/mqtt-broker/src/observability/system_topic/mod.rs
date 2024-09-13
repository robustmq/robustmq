// Copyright 2023 RobustMQ Team
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


use crate::{
    handler::{cache::CacheManager, topic::try_init_topic},
    storage::message::MessageStorage,
};
use broker::report_broker_info;
use clients::poll::ClientPool;
use common_base::tools::get_local_ip;
use log::{debug, error};
use metadata_struct::adapter::record::Record;
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};

// Cluster status information
pub const SYSTEM_TOPIC_BROKERS: &str = "$SYS/brokers";
pub const SYSTEM_TOPIC_BROKERS_VERSION: &str = "$SYS/brokers/${node}/version";
pub const SYSTEM_TOPIC_BROKERS_UPTIME: &str = "$SYS/brokers/${node}/uptime";
pub const SYSTEM_TOPIC_BROKERS_DATETIME: &str = "$SYS/brokers/${node}/datetime";
pub const SYSTEM_TOPIC_BROKERS_SYSDESCR: &str = "$SYS/brokers/${node}/sysdescr";

// Event
pub const SYSTEM_TOPIC_BROKERS_CONNECTED: &str =
    "$SYS/brokers/${node}/clients/${clientid}/connected";
pub const SYSTEM_TOPIC_BROKERS_DISCONNECTED: &str =
    "$SYS/brokers/${node}/clients/${clientid}/disconnected";
pub const SYSTEM_TOPIC_BROKERS_SUBSCRIBED: &str =
    "$SYS/brokers/${node}/clients/${clientid}/subscribed";
pub const SYSTEM_TOPIC_BROKERS_UNSUBSCRIBED: &str =
    "$SYS/brokers/${node}/clients/${clientid}/unsubscribed";

pub mod broker;
pub mod event;
pub mod packet;
pub mod stats;
pub mod sysmon;
pub mod warn;

pub struct SystemTopic<S> {
    pub metadata_cache: Arc<CacheManager>,
    pub message_storage_adapter: Arc<S>,
    pub client_poll: Arc<ClientPool>,
}

impl<S> SystemTopic<S>
where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    pub fn new(
        metadata_cache: Arc<CacheManager>,
        message_storage_adapter: Arc<S>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        return SystemTopic {
            metadata_cache,
            message_storage_adapter,
            client_poll,
        };
    }

    pub async fn start_thread(&self, stop_send: broadcast::Sender<bool>) {
        self.try_init_system_topic().await;
        let mut stop_rx = stop_send.subscribe();
        loop {
            select! {
                val = stop_rx.recv() =>{
                    match val{
                        Ok(flag) => {
                            if flag {
                                debug!("System topic thread stopped successfully");
                                break;
                            }
                        }
                        Err(_) => {}
                    }
                }
                _ = self.report_info()=>{}
            }
            sleep(Duration::from_secs(60)).await;
        }
    }

    pub async fn report_info(&self) {
        report_broker_info(
            &self.client_poll,
            &self.metadata_cache,
            &self.message_storage_adapter,
        )
        .await;
    }

    pub async fn try_init_system_topic(&self) {
        let results = self.get_all_system_topic();
        for topic_name in results {
            let new_topic_name = replace_topic_name(topic_name);
            match try_init_topic(
                &new_topic_name,
                &self.metadata_cache,
                &self.message_storage_adapter,
                &self.client_poll,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    panic!(
                        "Initializing system topic {} Failed, error message :{}",
                        new_topic_name,
                        e.to_string()
                    );
                }
            }
        }
    }

    fn get_all_system_topic(&self) -> Vec<String> {
        let mut results = Vec::new();
        results.push(SYSTEM_TOPIC_BROKERS.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_VERSION.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_UPTIME.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_DATETIME.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_SYSDESCR.to_string());
        return results;
    }
}

fn replace_topic_name(mut topic_name: String) -> String {
    if topic_name.contains("${node}") {
        let local_ip = get_local_ip();
        topic_name = topic_name.replace("${node}", &local_ip)
    }
    return topic_name;
}

pub(crate) async fn write_topic_data<S>(
    message_storage_adapter: &Arc<S>,
    metadata_cache: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    topic_name: String,
    record: Record,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    match try_init_topic(
        &topic_name,
        &metadata_cache.clone(),
        &message_storage_adapter.clone(),
        &client_poll.clone(),
    )
    .await
    {
        Ok(topic) => {
            let message_storage = MessageStorage::new(message_storage_adapter.clone());
            match message_storage
                .append_topic_message(topic.topic_id, vec![record])
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "Message written to system subject {} Error, error message :{}",
                        topic_name,
                        e.to_string()
                    );
                }
            }
        }
        Err(e) => {
            error!(
                "Initializing system topic {} Failed, error message :{}",
                topic_name,
                e.to_string()
            );
            return;
        }
    };
}
