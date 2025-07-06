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

use std::collections::HashMap;
use std::sync::Arc;

use common_base::tools::{get_local_ip, now_mills};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::mqtt::session::MqttSession;
use protocol::mqtt::common::{DisconnectReasonCode, MqttProtocol, Subscribe, Unsubscribe};
use serde::{Deserialize, Serialize};
use storage_adapter::storage::StorageAdapter;
use tracing::error;

use super::{
    write_topic_data, SYSTEM_TOPIC_BROKERS_CONNECTED, SYSTEM_TOPIC_BROKERS_DISCONNECTED,
    SYSTEM_TOPIC_BROKERS_SUBSCRIBED, SYSTEM_TOPIC_BROKERS_UNSUBSCRIBED,
};
use crate::handler::cache::CacheManager;
use crate::server::connection_manager::ConnectionManager;

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicConnectedEventMessage {
    pub username: String,
    pub ts: u128,
    pub sock_port: u16,
    pub proto_ver: Option<MqttProtocol>,
    pub proto_name: String,
    pub keepalive: u16,
    pub ip_address: String,
    pub expiry_interval: u64,
    pub connected_at: u128,
    pub connect_ack: u16,
    pub client_id: String,
    pub clean_start: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicDisConnectedEventMessage {
    pub username: String,
    pub ts: u128,
    pub sock_port: u16,
    pub reason: String,
    pub proto_ver: Option<MqttProtocol>,
    pub proto_name: String,
    pub ip_address: String,
    pub disconnected_at: u128,
    pub client_id: String,
}
#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicSubscribedEventMessage {
    pub username: String,
    pub ts: u128,
    pub subopts: SystemTopicSubscribedEventMessageSUbopts,
    pub topic: String,
    pub protocol: String,
    pub client_id: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicSubscribedEventMessageSUbopts {
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
    pub client_id: String,
}

// Go live event. When any client comes online, messages for that topic will be published
pub async fn st_report_connected_event<S>(
    message_storage_adapter: &Arc<S>,
    metadata_cache: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    session: &MqttSession,
    connection: &MQTTConnection,
    connect_id: u64,
    connection_manager: &Arc<ConnectionManager>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    if let Some(network_connection) = connection_manager.get_connect(connect_id) {
        let event_data = SystemTopicConnectedEventMessage {
            username: connection.login_user.clone(),
            ts: now_mills(),
            sock_port: network_connection.addr.port(),
            proto_ver: network_connection.protocol.clone(),
            proto_name: "MQTT".to_string(),
            keepalive: connection.keep_alive,
            ip_address: connection.source_ip_addr.clone(),
            expiry_interval: session.session_expiry,
            connected_at: now_mills(),
            connect_ack: 1,
            client_id: session.client_id.to_string(),
            clean_start: false,
        };
        match serde_json::to_string(&event_data) {
            Ok(data) => {
                let topic_name = replace_name(
                    SYSTEM_TOPIC_BROKERS_CONNECTED.to_string(),
                    session.client_id.to_string(),
                );

                if let Some(record) =
                    MqttMessage::build_system_topic_message(topic_name.clone(), data)
                {
                    write_topic_data(
                        message_storage_adapter,
                        metadata_cache,
                        client_pool,
                        topic_name,
                        record,
                    )
                    .await;
                }
            }
            Err(e) => {
                error!("{}", e.to_string());
            }
        }
    }
}

// Offline events. When any client goes offline, a message for that topic is published
#[allow(clippy::too_many_arguments)]
pub async fn st_report_disconnected_event<S>(
    message_storage_adapter: &Arc<S>,
    metadata_cache: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    session: &MqttSession,
    connection: &MQTTConnection,
    connect_id: u64,
    connection_manager: &Arc<ConnectionManager>,
    reason: Option<DisconnectReasonCode>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    if let Some(network_connection) = connection_manager.get_connect(connect_id) {
        let event_data = SystemTopicDisConnectedEventMessage {
            username: connection.login_user.clone(),
            ts: now_mills(),
            sock_port: network_connection.addr.port(),
            reason: format!("{reason:?}"),
            proto_ver: network_connection.protocol.clone(),
            proto_name: "MQTT".to_string(),
            ip_address: connection.source_ip_addr.clone(),
            client_id: session.client_id.to_string(),
            disconnected_at: now_mills(),
        };

        match serde_json::to_string(&event_data) {
            Ok(data) => {
                let topic_name = replace_name(
                    SYSTEM_TOPIC_BROKERS_DISCONNECTED.to_string(),
                    session.client_id.to_string(),
                );

                if let Some(record) =
                    MqttMessage::build_system_topic_message(topic_name.clone(), data)
                {
                    write_topic_data(
                        message_storage_adapter,
                        metadata_cache,
                        client_pool,
                        topic_name,
                        record,
                    )
                    .await;
                }
            }
            Err(e) => {
                error!("{}", e.to_string());
            }
        }
    }
}

// Subscribe to events. When any client subscribes to a topic, messages for that topic are published
pub async fn st_report_subscribed_event<S>(
    message_storage_adapter: &Arc<S>,
    metadata_cache: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    connection: &MQTTConnection,
    connect_id: u64,
    connection_manager: &Arc<ConnectionManager>,
    subscribe: &Subscribe,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    if let Some(network_connection) = connection_manager.get_connect(connect_id) {
        for filter in subscribe.filters.clone() {
            let subopts = SystemTopicSubscribedEventMessageSUbopts {
                sub_props: HashMap::new(),
                rh: if filter.preserve_retain { 1 } else { 0 },
                rap: filter.retain_handling.into(),
                qos: filter.qos.into(),
                nl: if filter.nolocal { 1 } else { 0 },
                is_new: true,
            };
            let event_data = SystemTopicSubscribedEventMessage {
                username: connection.login_user.clone(),
                ts: now_mills(),
                subopts,
                topic: filter.path,
                protocol: format!("{:?}", network_connection.protocol.clone()),
                client_id: connection.client_id.to_string(),
            };
            match serde_json::to_string(&event_data) {
                Ok(data) => {
                    let topic_name = replace_name(
                        SYSTEM_TOPIC_BROKERS_SUBSCRIBED.to_string(),
                        connection.client_id.to_string(),
                    );

                    if let Some(record) =
                        MqttMessage::build_system_topic_message(topic_name.clone(), data)
                    {
                        write_topic_data(
                            message_storage_adapter,
                            metadata_cache,
                            client_pool,
                            topic_name,
                            record,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    error!("{}", e.to_string());
                }
            }
        }
    }
}

// Unsubscribe from an event. When any client unsubscribes to a topic, messages for that topic are published
pub async fn st_report_unsubscribed_event<S>(
    message_storage_adapter: &Arc<S>,
    metadata_cache: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    connection: &MQTTConnection,
    connect_id: u64,
    connection_manager: &Arc<ConnectionManager>,
    un_subscribe: &Unsubscribe,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    if let Some(network_connection) = connection_manager.get_connect(connect_id) {
        for path in un_subscribe.filters.clone() {
            let event_data = SystemTopicUnSubscribedEventMessage {
                username: connection.login_user.clone(),
                ts: now_mills(),
                topic: path,
                protocol: format!("{:?}", network_connection.protocol.clone()),
                client_id: connection.client_id.to_string(),
            };
            match serde_json::to_string(&event_data) {
                Ok(data) => {
                    let topic_name = replace_name(
                        SYSTEM_TOPIC_BROKERS_UNSUBSCRIBED.to_string(),
                        connection.client_id.to_string(),
                    );

                    if let Some(record) =
                        MqttMessage::build_system_topic_message(topic_name.clone(), data)
                    {
                        write_topic_data(
                            message_storage_adapter,
                            metadata_cache,
                            client_pool,
                            topic_name,
                            record,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    error!("{}", e.to_string());
                }
            }
        }
    }
}

fn replace_name(mut topic_name: String, client_id: String) -> String {
    if topic_name.contains("${node}") {
        let local_ip = get_local_ip();
        topic_name = topic_name.replace("${node}", &local_ip)
    }
    if topic_name.contains("${client_id}") {
        topic_name = topic_name.replace("${client_id}", &client_id)
    }
    topic_name
}
