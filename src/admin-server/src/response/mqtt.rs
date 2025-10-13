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

use metadata_struct::{
    connection::NetworkConnection,
    mqtt::{connection::MQTTConnection, session::MqttSession, topic::MQTTTopic},
    placement::node::BrokerNode,
};
use mqtt_broker::{handler::cache::ConnectionLiveTime, subscribe::manager::TopicSubscribeInfo};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientListRow {
    pub client_id: String,
    pub connection_id: u64,
    pub mqtt_connection: MQTTConnection,
    pub network_connection: Option<NetworkConnection>,
    pub session: Option<MqttSession>,
    pub heartbeat: Option<ConnectionLiveTime>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicListRow {
    pub topic_name: String,
    pub create_time: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicDetailResp {
    pub topic_info: MQTTTopic,
    pub retain_message: Option<String>,
    pub retain_message_at: Option<u64>,
    pub sub_list: Vec<TopicSubscribeInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicRewriteListRow {
    pub source_topic: String,
    pub dest_topic: String,
    pub regex: String,
    pub action: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserListRow {
    pub username: String,
    pub is_superuser: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AclListRow {
    pub resource_type: String,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: String,
    pub permission: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlackListListRow {
    pub blacklist_type: String,
    pub resource_name: String,
    pub end_time: String,
    pub desc: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectorListRow {
    pub connector_name: String,
    pub connector_type: String,
    pub config: String,
    pub topic_name: String,
    pub status: String,
    pub broker_id: String,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaListRow {
    pub name: String,
    pub schema_type: String,
    pub desc: String,
    pub schema: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaBindListRow {
    pub data_type: String,
    pub data: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubscribeListRow {
    pub client_id: String,
    pub path: String,
    pub broker_id: u64,
    pub protocol: String,
    pub qos: String,
    pub no_local: u32,
    pub preserve_retain: u32,
    pub retain_handling: String,
    pub create_time: String,
    pub pk_id: u32,
    pub properties: String,
    pub is_share_sub: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AutoSubscribeListRow {
    pub topic: String,
    pub qos: String,
    pub no_local: bool,
    pub retain_as_published: bool,
    pub retained_handling: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SlowSubscribeListRow {
    pub client_id: String,
    pub topic_name: String,
    pub time_span: u64,
    pub node_info: String,
    pub create_time: String,
    pub subscribe_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionListRow {
    pub client_id: String,
    pub session_expiry: u64,
    pub is_contain_last_will: bool,
    pub last_will_delay_interval: Option<u64>,
    pub create_time: u64,
    pub connection_id: Option<u64>,
    pub broker_id: Option<u64>,
    pub reconnect_time: Option<u64>,
    pub distinct_time: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SystemAlarmListRow {
    pub name: String,
    pub message: String,
    pub create_time: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FlappingDetectListRaw {
    pub client_id: String,
    pub before_last_windows_connections: u64,
    pub first_request_time: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BanLogListRaw {
    pub ban_type: String,
    pub resource_name: String,
    pub ban_source: String,
    pub end_time: String,
    pub create_time: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OverViewResp {
    pub node_list: Vec<BrokerNode>,
    pub cluster_name: String,
    pub message_in_rate: u64,
    pub message_out_rate: u64,
    pub connection_num: u32,
    pub session_num: u32,
    pub topic_num: u32,
    pub placement_status: String,
    pub tcp_connection_num: u32,
    pub tls_connection_num: u32,
    pub websocket_connection_num: u32,
    pub quic_connection_num: u32,
    pub subscribe_num: u32,
    pub exclusive_subscribe_num: u32,
    pub share_subscribe_leader_num: u32,
    pub share_subscribe_resub_num: u32,
    pub exclusive_subscribe_thread_num: u32,
    pub share_subscribe_leader_thread_num: u32,
    pub share_subscribe_follower_thread_num: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OverViewMetricsResp {
    pub connection_num: Vec<HashMap<String, u64>>,
    pub topic_num: Vec<HashMap<String, u64>>,
    pub subscribe_num: Vec<HashMap<String, u64>>,
    pub message_in_num: Vec<HashMap<String, u64>>,
    pub message_out_num: Vec<HashMap<String, u64>>,
    pub message_drop_num: Vec<HashMap<String, u64>>,
}
