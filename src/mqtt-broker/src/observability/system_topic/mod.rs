use crate::{
    handler::{cache::CacheManager, topic::try_init_topic},
    storage::message::MessageStorage,
};
use broker::report_cluster_status;
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
pub const SYSTEM_TOPIC_BROKERS_CLIENTS: &str = "$SYS/brokers/${node}/clients";

pub mod broker;
pub mod event;
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
                _ = self.report_info()=>{

                }
            }

            sleep(Duration::from_secs(60)).await;
        }
    }

    pub async fn report_info(&self) {
        report_cluster_status(
            self.client_poll.clone(),
            self.metadata_cache.clone(),
            self.message_storage_adapter.clone(),
        )
        .await;
    }

    pub async fn try_init_system_topic(&self) {
        let results = self.get_all_system_topic();
        for topic_name in results {
            let new_topic_name = self.replace_topic_name(topic_name);
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
    fn replace_topic_name(&self, mut topic_name: String) -> String {
        if topic_name.contains("${node}") {
            let local_ip = get_local_ip();
            topic_name = topic_name.replace("${node}", &local_ip)
        }
        return topic_name;
    }

    fn get_all_system_topic(&self) -> Vec<String> {
        let mut results = Vec::new();
        results.push(SYSTEM_TOPIC_BROKERS.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_VERSION.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_UPTIME.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_DATETIME.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_SYSDESCR.to_string());
        results.push(SYSTEM_TOPIC_BROKERS_CLIENTS.to_string());
        return results;
    }
}

pub(crate) async fn write_topic_data<S>(
    message_storage_adapter: Arc<S>,
    topic_name: String,
    topic_id: String,
    record: Record,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    let message_storage = MessageStorage::new(message_storage_adapter.clone());
    match message_storage
        .append_topic_message(topic_id, vec![record])
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
