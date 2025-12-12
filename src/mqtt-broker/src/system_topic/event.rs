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

use common_base::tools::{get_local_ip, now_millis};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::mqtt::session::MqttSession;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::common::{DisconnectReasonCode, MqttProtocol, Subscribe, Unsubscribe};
use serde::{Deserialize, Serialize};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;

use super::{
    write_topic_data, SYSTEM_TOPIC_BROKERS_CONNECTED, SYSTEM_TOPIC_BROKERS_DISCONNECTED,
    SYSTEM_TOPIC_BROKERS_SUBSCRIBED, SYSTEM_TOPIC_BROKERS_UNSUBSCRIBED,
};
use crate::handler::cache::MQTTCacheManager;

#[derive(Clone)]
pub struct StReportDisconnectedEventContext {
    pub message_storage_adapter: ArcStorageAdapter,
    pub metadata_cache: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub session: MqttSession,
    pub connection: MQTTConnection,
    pub connect_id: u64,
    pub connection_manager: Arc<ConnectionManager>,
    pub reason: Option<DisconnectReasonCode>,
}

#[derive(Clone)]
pub struct StReportSubscribedEventContext {
    pub message_storage_adapter: ArcStorageAdapter,
    pub metadata_cache: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub connection: MQTTConnection,
    pub connect_id: u64,
    pub connection_manager: Arc<ConnectionManager>,
    pub subscribe: Subscribe,
}

#[derive(Clone)]
pub struct StReportUnsubscribedEventContext {
    pub message_storage_adapter: ArcStorageAdapter,
    pub metadata_cache: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub connection: MQTTConnection,
    pub connect_id: u64,
    pub connection_manager: Arc<ConnectionManager>,
    pub un_subscribe: Unsubscribe,
}

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

#[derive(Clone)]
pub struct StReportConnectedEventContext {
    pub message_storage_adapter: ArcStorageAdapter,
    pub metadata_cache: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub session: MqttSession,
    pub connection: MQTTConnection,
    pub connect_id: u64,
    pub connection_manager: Arc<ConnectionManager>,
}

// Go live event. When any client comes online, messages for that topic will be published
pub async fn st_report_connected_event(context: StReportConnectedEventContext) {
    if let Some(network_connection) = context.connection_manager.get_connect(context.connect_id) {
        let event_data = SystemTopicConnectedEventMessage {
            username: context.connection.login_user.clone(),
            ts: now_millis(),
            sock_port: network_connection.addr.port(),
            proto_ver: Some(network_connection.protocol.unwrap().to_mqtt()),
            proto_name: "MQTT".to_string(),
            keepalive: context.connection.keep_alive,
            ip_address: context.connection.source_ip_addr.clone(),
            expiry_interval: context.session.session_expiry,
            connected_at: now_millis(),
            connect_ack: 1,
            client_id: context.session.client_id.to_string(),
            clean_start: false,
        };
        match serde_json::to_string(&event_data) {
            Ok(data) => {
                let topic_name = replace_name(
                    SYSTEM_TOPIC_BROKERS_CONNECTED.to_string(),
                    context.session.client_id.to_string(),
                );

                if let Some(record) =
                    MqttMessage::build_system_topic_message(topic_name.clone(), data)
                {
                    let _ = write_topic_data(
                        &context.message_storage_adapter,
                        &context.metadata_cache,
                        &context.client_pool,
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
pub async fn st_report_disconnected_event(context: StReportDisconnectedEventContext) {
    if let Some(network_connection) = context.connection_manager.get_connect(context.connect_id) {
        let event_data = SystemTopicDisConnectedEventMessage {
            username: context.connection.login_user.clone(),
            ts: now_millis(),
            sock_port: network_connection.addr.port(),
            reason: format!("{:?}", context.reason),
            proto_ver: Some(network_connection.protocol.unwrap().to_mqtt()),
            proto_name: "MQTT".to_string(),
            ip_address: context.connection.source_ip_addr.clone(),
            client_id: context.session.client_id.to_string(),
            disconnected_at: now_millis(),
        };

        match serde_json::to_string(&event_data) {
            Ok(data) => {
                let topic_name = replace_name(
                    SYSTEM_TOPIC_BROKERS_DISCONNECTED.to_string(),
                    context.session.client_id.to_string(),
                );

                if let Some(record) =
                    MqttMessage::build_system_topic_message(topic_name.clone(), data)
                {
                    let _ = write_topic_data(
                        &context.message_storage_adapter,
                        &context.metadata_cache,
                        &context.client_pool,
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
pub async fn st_report_subscribed_event(context: StReportSubscribedEventContext) {
    if let Some(network_connection) = context.connection_manager.get_connect(context.connect_id) {
        for filter in context.subscribe.filters.clone() {
            let subopts = SystemTopicSubscribedEventMessageSUbopts {
                sub_props: HashMap::new(),
                rh: if filter.preserve_retain { 1 } else { 0 },
                rap: filter.retain_handling.into(),
                qos: filter.qos.into(),
                nl: if filter.nolocal { 1 } else { 0 },
                is_new: true,
            };
            let event_data = SystemTopicSubscribedEventMessage {
                username: context.connection.login_user.clone(),
                ts: now_millis(),
                subopts,
                topic: filter.path,
                protocol: format!("{:?}", network_connection.protocol),
                client_id: context.connection.client_id.to_string(),
            };
            match serde_json::to_string(&event_data) {
                Ok(data) => {
                    let topic_name = replace_name(
                        SYSTEM_TOPIC_BROKERS_SUBSCRIBED.to_string(),
                        context.connection.client_id.to_string(),
                    );

                    if let Some(record) =
                        MqttMessage::build_system_topic_message(topic_name.clone(), data)
                    {
                        let _ = write_topic_data(
                            &context.message_storage_adapter,
                            &context.metadata_cache,
                            &context.client_pool,
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
pub async fn st_report_unsubscribed_event(context: StReportUnsubscribedEventContext) {
    if let Some(network_connection) = context.connection_manager.get_connect(context.connect_id) {
        for path in context.un_subscribe.filters.clone() {
            let event_data = SystemTopicUnSubscribedEventMessage {
                username: context.connection.login_user.clone(),
                ts: now_millis(),
                topic: path,
                protocol: format!("{:?}", network_connection.protocol),
                client_id: context.connection.client_id.to_string(),
            };
            match serde_json::to_string(&event_data) {
                Ok(data) => {
                    let topic_name = replace_name(
                        SYSTEM_TOPIC_BROKERS_UNSUBSCRIBED.to_string(),
                        context.connection.client_id.to_string(),
                    );

                    if let Some(record) =
                        MqttMessage::build_system_topic_message(topic_name.clone(), data)
                    {
                        let _ = write_topic_data(
                            &context.message_storage_adapter,
                            &context.metadata_cache,
                            &context.client_pool,
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
