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

use metadata_struct::placement::node::BrokerNode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PageReplyData<T> {
    pub data: T,
    pub total_count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientListRow {
    pub connection_id: u64,
    pub connection_type: String,
    pub protocol: String,
    pub source_addr: String,
    pub create_time: String,
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
pub struct OverViewResp {
    pub node_list: Vec<BrokerNode>,
    pub cluster_name: String,
    pub message_in_rate: u32,
    pub message_out_rate: u32,
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
    pub connection_num: String,
    pub topic_num: String,
    pub subscribe_num: String,
    pub message_in_num: String,
    pub message_out_num: String,
    pub message_drop_num: String,
}
