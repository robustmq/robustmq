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

use crate::core::cache::MQTTCacheManager;
use crate::core::topic::try_init_topic;
use crate::storage::message::MessageStorage;
use crate::system_topic::build_system_topic_payload;
use common_base::tools::now_millis;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::tenant::DEFAULT_TENANT;
use metadata_struct::{
    mqtt::connection::MQTTConnection, storage::adapter_record::AdapterWriteRecord,
};
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::common::{DisconnectReasonCode, Subscribe, Unsubscribe};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

// Event
pub const SYSTEM_TOPIC_EVENT: &str = "$SYS/events";

const EVENT_CHANNEL_SIZE: usize = 2000;
const EVENT_BATCH_SIZE: usize = 100;

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicConnectedEventMessage {
    pub username: String,
    pub ts: u128,
    #[serde(rename = "sockport")]
    pub sock_port: u16,
    pub proto_ver: u8,
    pub proto_name: String,
    pub keepalive: u16,
    #[serde(rename = "ipaddress")]
    pub ip_address: String,
    pub expiry_interval: u64,
    pub connected_at: u128,
    #[serde(rename = "connack")]
    pub connect_ack: u16,
    #[serde(rename = "clientid")]
    pub client_id: String,
    pub clean_start: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicDisConnectedEventMessage {
    pub username: String,
    pub ts: u128,
    #[serde(rename = "sockport")]
    pub sock_port: u16,
    pub reason: String,
    pub proto_ver: u8,
    pub proto_name: String,
    #[serde(rename = "ipaddress")]
    pub ip_address: String,
    pub disconnected_at: u128,
    #[serde(rename = "clientid")]
    pub client_id: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicSubscribedEventMessage {
    pub username: String,
    pub ts: u128,
    pub subopts: SystemTopicSubscribedEventMessageSupports,
    pub topic: String,
    pub protocol: String,
    #[serde(rename = "clientid")]
    pub client_id: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicSubscribedEventMessageSupports {
    pub sub_props: HashMap<String, String>,
    pub rh: u16,
    pub rap: u8,
    pub qos: u8,
    pub nl: u16,
    pub is_new: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicUnSubscribedEventMessage {
    pub username: String,
    pub ts: u128,
    pub topic: String,
    pub protocol: String,
    #[serde(rename = "clientid")]
    pub client_id: String,
}

pub enum EventData {
    Connected(SystemTopicConnectedEventMessage),
    Disconnected(SystemTopicDisConnectedEventMessage),
    Subscribed(SystemTopicSubscribedEventMessage),
    Unsubscribed(SystemTopicUnSubscribedEventMessage),
}

pub struct EventMessage {
    pub data: EventData,
}

pub struct EventReportManager {
    tx: mpsc::Sender<EventMessage>,
    consumer: std::sync::Mutex<Option<mpsc::Receiver<EventMessage>>>,
}

impl EventReportManager {
    pub fn new() -> Arc<Self> {
        let (tx, rx) = mpsc::channel(EVENT_CHANNEL_SIZE);
        Arc::new(EventReportManager {
            tx,
            consumer: std::sync::Mutex::new(Some(rx)),
        })
    }

    pub async fn start(
        &self,
        cache_manager: Arc<MQTTCacheManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        client_pool: Arc<ClientPool>,
    ) {
        let rx = self
            .consumer
            .lock()
            .unwrap()
            .take()
            .expect("EventReportManager::start must be called exactly once");
        event_batch_consumer(rx, cache_manager, storage_driver_manager, client_pool).await;
    }

    pub async fn report(&self, msg: EventMessage) {
        if let Err(e) = self.tx.send(msg).await {
            error!("report event channel closed: {}", e);
        }
    }
}

async fn event_batch_consumer(
    mut rx: mpsc::Receiver<EventMessage>,
    cache_manager: Arc<MQTTCacheManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    client_pool: Arc<ClientPool>,
) {
    let topic = match try_init_topic(
        DEFAULT_TENANT,
        SYSTEM_TOPIC_EVENT,
        &cache_manager,
        &storage_driver_manager,
        &client_pool,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            panic!(
                "EventReportManager: failed to init topic {}: {}",
                SYSTEM_TOPIC_EVENT, e
            );
        }
    };

    let message_storage = MessageStorage::new(storage_driver_manager.clone());

    loop {
        let first = match rx.recv().await {
            Some(item) => item,
            None => {
                info!("EventReportManager channel closed, consumer stopping");
                return;
            }
        };

        let mut batch = vec![first];

        loop {
            if batch.len() >= EVENT_BATCH_SIZE {
                break;
            }
            match rx.try_recv() {
                Ok(item) => batch.push(item),
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    info!("EventReportManager channel closed during batch collection");
                    flush_batch(batch, &topic.topic_name, &message_storage).await;
                    return;
                }
            }
        }

        flush_batch(batch, &topic.topic_name, &message_storage).await;
    }
}

async fn flush_batch(batch: Vec<EventMessage>, topic_name: &str, message_storage: &MessageStorage) {
    let mut records = Vec::with_capacity(batch.len());

    for msg in batch {
        let payload = match serialize_event_data(msg.data) {
            Ok(p) => p,
            Err(e) => {
                warn!("EventReportManager: failed to serialize event: {}", e);
                continue;
            }
        };

        let record = AdapterWriteRecord::new(topic_name.to_string(), payload);
        records.push(record);
    }

    if records.is_empty() {
        return;
    }

    if let Err(e) = message_storage
        .append_topic_message(DEFAULT_TENANT, topic_name, records)
        .await
    {
        warn!(
            "EventReportManager: failed to write events to {}: {}",
            topic_name, e
        );
    }
}

fn serialize_event_data(data: EventData) -> Result<String, serde_json::Error> {
    match data {
        EventData::Connected(d) => build_system_topic_payload(d),
        EventData::Disconnected(d) => build_system_topic_payload(d),
        EventData::Subscribed(d) => build_system_topic_payload(d),
        EventData::Unsubscribed(d) => build_system_topic_payload(d),
    }
}

pub async fn st_report_connected_event(
    event_manager: &Arc<EventReportManager>,
    connection_manager: &Arc<ConnectionManager>,
    connect_id: u64,
    connection: &MQTTConnection,
    session: &MqttSession,
) {
    if let Some(network_connection) = connection_manager.get_connect(connect_id) {
        let event_data = SystemTopicConnectedEventMessage {
            username: connection.login_user.clone().unwrap_or_default(),
            ts: now_millis(),
            sock_port: network_connection.addr.port(),
            proto_ver: network_connection
                .protocol
                .as_ref()
                .map(|p| p.to_u8())
                .unwrap_or(0),
            proto_name: "MQTT".to_string(),
            keepalive: connection.keep_alive,
            ip_address: connection.source_ip_addr.clone(),
            expiry_interval: session.session_expiry_interval,
            connected_at: now_millis(),
            connect_ack: 0,
            client_id: session.client_id.to_string(),
            clean_start: !session.is_persist_session,
        };
        event_manager
            .report(EventMessage {
                data: EventData::Connected(event_data),
            })
            .await;
    }
}

pub async fn st_report_disconnected_event(
    event_manager: &Arc<EventReportManager>,
    connection_manager: &Arc<ConnectionManager>,
    connect_id: u64,
    connection: &MQTTConnection,
    session: &MqttSession,
    reason: Option<DisconnectReasonCode>,
) {
    if let Some(network_connection) = connection_manager.get_connect(connect_id) {
        let event_data = SystemTopicDisConnectedEventMessage {
            username: connection.login_user.clone().unwrap_or_default(),
            ts: now_millis(),
            sock_port: network_connection.addr.port(),
            reason: format!("{:?}", reason),
            proto_ver: network_connection
                .protocol
                .as_ref()
                .map(|p| p.to_u8())
                .unwrap_or(0),
            proto_name: "MQTT".to_string(),
            ip_address: connection.source_ip_addr.clone(),
            client_id: session.client_id.to_string(),
            disconnected_at: now_millis(),
        };
        event_manager
            .report(EventMessage {
                data: EventData::Disconnected(event_data),
            })
            .await;
    }
}

pub async fn st_report_subscribed_event(
    event_manager: &Arc<EventReportManager>,
    connection_manager: &Arc<ConnectionManager>,
    connect_id: u64,
    connection: &MQTTConnection,
    subscribe: &Subscribe,
) {
    let username = connection.login_user.clone().unwrap_or_default();
    if let Some(network_connection) = connection_manager.get_connect(connect_id) {
        for filter in subscribe.filters.iter() {
            let subopts = SystemTopicSubscribedEventMessageSupports {
                sub_props: HashMap::new(),
                rh: if filter.preserve_retain { 1 } else { 0 },
                rap: filter.retain_handling.clone().into(),
                qos: filter.qos.into(),
                nl: if filter.no_local { 1 } else { 0 },
                is_new: true,
            };
            let event_data = SystemTopicSubscribedEventMessage {
                username: username.clone(),
                ts: now_millis(),
                subopts,
                topic: filter.path.clone(),
                protocol: format!("{:?}", network_connection.protocol),
                client_id: connection.client_id.to_string(),
            };
            event_manager
                .report(EventMessage {
                    data: EventData::Subscribed(event_data),
                })
                .await;
        }
    }
}

pub async fn st_report_unsubscribed_event(
    event_manager: &Arc<EventReportManager>,
    connection_manager: &Arc<ConnectionManager>,
    connect_id: u64,
    connection: &MQTTConnection,
    un_subscribe: &Unsubscribe,
) {
    let username = connection.login_user.clone().unwrap_or_default();
    if let Some(network_connection) = connection_manager.get_connect(connect_id) {
        for path in un_subscribe.filters.iter() {
            let event_data = SystemTopicUnSubscribedEventMessage {
                username: username.clone(),
                ts: now_millis(),
                topic: path.clone(),
                protocol: format!("{:?}", network_connection.protocol),
                client_id: connection.client_id.to_string(),
            };
            event_manager
                .report(EventMessage {
                    data: EventData::Unsubscribed(event_data),
                })
                .await;
        }
    }
}
