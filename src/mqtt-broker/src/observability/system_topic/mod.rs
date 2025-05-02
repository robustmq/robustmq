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

use std::sync::Arc;
use std::time::Duration;

use broker::report_broker_info;
use common_base::tools::get_local_ip;
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::record::Record;
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error};

use crate::handler::cache::CacheManager;
use crate::handler::topic::try_init_topic;
use crate::storage::message::MessageStorage;

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

// Stats
// connections
pub const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/connections/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX: &str =
    "$SYS/brokers/${node}/stats/connections/max";
// subscribe
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/suboptions/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX: &str =
    "$SYS/brokers/${node}/stats/suboptions/max";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscribers/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscriptions/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscriptions/shared/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX: &str =
    "$SYS/brokers/${node}/stats/subscriptions/shared/max";

pub const SYSTEM_TOPIC_BROKERS_STATS_TOPICS_COUNT: &str = "$SYS/brokers/${node}/stats/topics/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_TOPICS_MAX: &str = "$SYS/brokers/${node}/stats/topics/max";
pub const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT: &str = "$SYS/brokers/${node}/stats/routes/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX: &str = "$SYS/brokers/${node}/stats/routes/max";

// metrics
pub const SYSTEM_TOPIC_BROKERS_METRICS_BYTES_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/bytes/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_BYTES_SENT: &str = "$SYS/brokers/${node}/metrics/bytes/sent";

// MQTT Packet Received and Sent
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNECT: &str =
    "$SYS/brokers/${node}/metrics/packets/connect";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNACK: &str =
    "$SYS/brokers/${node}/metrics/packets/connack";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/publish/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/publish/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/missed";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/missed";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/missed";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/missed";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBSCRIBE: &str =
    "$SYS/brokers/${node}/metrics/packets/subscribe";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBACK: &str =
    "$SYS/brokers/${node}/metrics/packets/suback";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBSCRIBE: &str =
    "$SYS/brokers/${node}/metrics/packets/unsubscribe";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBACK: &str =
    "$SYS/brokers/${node}/metrics/packets/unsuback";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGREQ: &str =
    "$SYS/brokers/${node}/metrics/packets/pingreq";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGRESP: &str =
    "$SYS/brokers/${node}/metrics/packets/pingresp";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/disconnect/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/disconnect/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_AUTH: &str =
    "$SYS/brokers/${node}/metrics/packets/auth";

// MQTT Message Received and Sent
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_EXPIRED: &str =
    "$SYS/brokers/${node}/metrics/messages/expired";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RETAINED: &str =
    "$SYS/brokers/${node}/metrics/messages/retained";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_DROPPED: &str =
    "$SYS/brokers/${node}/metrics/messages/dropped";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_FORWARD: &str =
    "$SYS/brokers/${node}/metrics/messages/forward";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos0/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos0/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos1/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos1/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_EXPIRED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/expired";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_DROPPED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/dropped";

// System alarm
pub const SYSTEM_TOPIC_BROKERS_ALARMS_ALERT: &str = "$SYS/brokers/${node}/alarms/alert";
pub const SYSTEM_TOPIC_BROKERS_ALARMS_CLEAR: &str = "$SYS/brokers/${node}/alarms/clear";

// system symon
pub const SYSTEM_TOPIC_BROKERS_SYSMON_LONG_GC: &str = "$SYS/brokers/${node}/sysmon/long_gc";
pub const SYSTEM_TOPIC_BROKERS_SYSMON_LONG_SCHEDULE: &str =
    "$SYS/brokers/${node}/sysmon/long_schedule";
pub const SYSTEM_TOPIC_BROKERS_SYSMON_LARGE_HEAP: &str = "$SYS/brokers/${node}/sysmon/large_heap";
pub const SYSTEM_TOPIC_BROKERS_SYSMON_BUSY_PORT: &str = "$SYS/brokers/${node}/sysmon/busy_port";
pub const SYSTEM_TOPIC_BROKERS_SYSMON_BUSY_DIST_PORT: &str =
    "$SYS/brokers/${node}/sysmon/busy_dist_port";

pub mod broker;
pub mod event;
pub mod packet;
pub mod stats;
pub mod sysmon;
pub mod warn;

pub struct SystemTopic<S> {
    pub metadata_cache: Arc<CacheManager>,
    pub message_storage_adapter: Arc<S>,
    pub client_pool: Arc<ClientPool>,
}

impl<S> SystemTopic<S>
where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    pub fn new(
        metadata_cache: Arc<CacheManager>,
        message_storage_adapter: Arc<S>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        SystemTopic {
            metadata_cache,
            message_storage_adapter,
            client_pool,
        }
    }

    pub async fn start_thread(&self, stop_send: broadcast::Sender<bool>) {
        self.try_init_system_topic().await;
        let mut stop_rx = stop_send.subscribe();
        loop {
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            debug!("System topic thread stopped successfully");
                            break;
                        }
                    }
                }
                _ = self.report_info()=>{}
            }
            sleep(Duration::from_secs(60)).await;
        }
    }

    pub async fn report_info(&self) {
        report_broker_info(
            &self.client_pool,
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
                &self.client_pool,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    panic!(
                        "Initializing system topic {} Failed, error message :{}",
                        new_topic_name, e
                    );
                }
            }
        }
    }

    fn get_all_system_topic(&self) -> Vec<String> {
        vec![
            SYSTEM_TOPIC_BROKERS.to_string(),
            SYSTEM_TOPIC_BROKERS_VERSION.to_string(),
            SYSTEM_TOPIC_BROKERS_UPTIME.to_string(),
            SYSTEM_TOPIC_BROKERS_DATETIME.to_string(),
            SYSTEM_TOPIC_BROKERS_SYSDESCR.to_string(),
        ]
    }
}

fn replace_topic_name(mut topic_name: String) -> String {
    if topic_name.contains("${node}") {
        let local_ip = get_local_ip();
        topic_name = topic_name.replace("${node}", &local_ip)
    }
    topic_name
}

pub(crate) async fn write_topic_data<S>(
    message_storage_adapter: &Arc<S>,
    metadata_cache: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    topic_name: String,
    record: Record,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    match try_init_topic(
        &topic_name,
        &metadata_cache.clone(),
        &message_storage_adapter.clone(),
        &client_pool.clone(),
    )
    .await
    {
        Ok(topic) => {
            let message_storage = MessageStorage::new(message_storage_adapter.clone());
            match message_storage
                .append_topic_message(&topic.topic_id, vec![record])
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
        }
    }
}
