/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RobustConfig {
    pub addr: String,
    pub grpc_port: usize,
    pub broker: Broker,
    pub network: Network,
    pub runtime: Runtime,
    pub mqtt: MQTT,
    pub log: Log,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Broker {
    pub port: Option<u16>,
    pub work_thread: Option<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Network {
    pub accept_thread_num: usize,
    pub handler_thread_num: usize,
    pub response_thread_num: usize,
    pub max_connection_num: usize,
    pub request_queue_size: usize,
    pub response_queue_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Runtime {
    pub data_worker_threads: usize,
    pub inner_worker_threads: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MQTT {
    pub mqtt4_port: usize,
    pub mqtt5_port: usize,
    pub websocket_port: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Log{
    pub log_path: String,
    pub log_segment_size: u64,
    pub log_file_num: u32,
}