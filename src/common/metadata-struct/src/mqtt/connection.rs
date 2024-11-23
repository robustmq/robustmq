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

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;

use common_base::tools::now_second;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MQTTConnection {
    // Connection ID
    pub connect_id: u64,
    // Each connection has a unique Client ID
    pub client_id: String,
    // Mark whether the link is already logged in
    pub is_login: bool,
    //
    pub source_ip_addr: String,
    //
    pub login_user: String,
    // When the client does not report a heartbeat, the maximum survival time of the connection,
    pub keep_alive: u16,
    // Records the Topic alias information for the connection dimension
    pub topic_alias: DashMap<u16, String>,
    // Record the maximum number of QOS1 and QOS2 packets that the client can send in connection dimension. Scope of data flow control.
    pub client_max_receive_maximum: u16,
    // Record the connection dimension, the size of the maximum request packet that can be received.
    pub max_packet_size: u32,
    // Record the maximum number of connection dimensions and topic aliases. The default value ranges from 0 to 65535
    pub topic_alias_max: u16,
    // Flags whether to return a detailed error message to the client when an error occurs.
    pub request_problem_info: u8,
    // Flow control part keeps track of how many QOS 1 and QOS 2 messages are still pending on the connection
    #[serde(skip_serializing, skip_deserializing)]
    pub receive_qos_message: Arc<AtomicIsize>,
    // Flow control part keeps track of how many QOS 1 and QOS 2 messages are still pending on the connection
    #[serde(skip_serializing, skip_deserializing)]
    pub sender_qos_message: Arc<AtomicIsize>,
    // Time when the connection was created
    pub create_time: u64,
}

pub struct ConnectionConfig {
    pub connect_id: u64,
    pub client_id: String,
    pub receive_maximum: u16,
    pub max_packet_size: u32,
    pub topic_alias_max: u16,
    pub request_problem_info: u8,
    pub keep_alive: u16,
    pub source_ip_addr: String,
}

impl MQTTConnection {
    pub fn new(config: ConnectionConfig) -> MQTTConnection {
        MQTTConnection {
            connect_id: config.connect_id,
            client_id: config.client_id,
            is_login: false,
            keep_alive: config.keep_alive,
            client_max_receive_maximum: config.receive_maximum,
            max_packet_size: config.max_packet_size,
            topic_alias: DashMap::with_capacity(2),
            topic_alias_max: config.topic_alias_max,
            request_problem_info: config.request_problem_info,
            receive_qos_message: Arc::new(AtomicIsize::new(0)),
            sender_qos_message: Arc::new(AtomicIsize::new(0)),
            create_time: now_second(),
            source_ip_addr: config.source_ip_addr,
            ..Default::default()
        }
    }

    pub fn login_success(&mut self, user_name: String) {
        self.is_login = true;
        self.login_user = user_name;
    }

    pub fn is_response_problem_info(&self) -> bool {
        self.request_problem_info == 1
    }

    pub fn get_recv_qos_message(&self) -> isize {
        self.receive_qos_message.fetch_add(0, Ordering::Relaxed)
    }

    pub fn recv_qos_message_incr(&self) {
        self.receive_qos_message.fetch_add(1, Ordering::Relaxed);
    }

    pub fn recv_qos_message_decr(&self) {
        self.receive_qos_message.fetch_add(-1, Ordering::Relaxed);
    }

    pub fn get_send_qos_message(&self) -> isize {
        self.sender_qos_message.fetch_add(0, Ordering::Relaxed)
    }

    pub fn send_qos_message_incr(&self) {
        self.sender_qos_message.fetch_add(1, Ordering::Relaxed);
    }

    pub fn send_qos_message_decr(&self) {
        self.sender_qos_message.fetch_add(-1, Ordering::Relaxed);
    }
}
