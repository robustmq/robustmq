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


use super::{
    broker_mqtt::{Log, Network, System, TcpThread},
    common::{Auth, Storage},
};

pub fn default_grpc_port() -> u32 {
    9981
}

pub fn default_http_port() -> usize {
    9982
}

pub fn default_network() -> Network {
    Network {
        tcp_port: default_network_tcp_port(),
        tcps_port: default_network_tcps_port(),
        websocket_port: default_network_websocket_port(),
        websockets_port: default_network_websockets_port(),
        quic_port: default_network_quic_port(),
        tls_cert: "".to_string(),
        tls_key: "".to_string(),
    }
}
pub fn default_network_tcp_port() -> u32 {
    1883
}
pub fn default_network_tcps_port() -> u32 {
    1884
}
pub fn default_network_websocket_port() -> u32 {
    8083
}
pub fn default_network_websockets_port() -> u32 {
    8084
}
pub fn default_network_quic_port() -> u32 {
    9083
}

pub fn default_tcp_thread() -> TcpThread {
    TcpThread {
        accept_thread_num: 1,
        handler_thread_num: 1,
        response_thread_num: 1,
        max_connection_num: 1000,
        request_queue_size: 2000,
        response_queue_size: 2000,
        lock_max_try_mut_times: 30,
        lock_try_mut_sleep_time_ms: 50,
    }
}

pub fn default_system() -> System {
    System {
        runtime_worker_threads: 16,
        default_user: "admin".to_string(),
        default_password: "robustmq".to_string(),
    }
}

pub fn default_storage() -> Storage {
    Storage {
        storage_type: "memory".to_string(),
        journal_addr: "".to_string(),
        mysql_addr: "".to_string(),
    }
}

pub fn default_log() -> Log {
    Log {
        log_path: format!("/tmp/robust-default"),
        log_segment_size: 1073741824,
        log_file_num: 50,
    }
}

pub fn default_auth() -> Auth {
    Auth {
        storage_type: "memory".to_string(),
        journal_addr: "".to_string(),
        mysql_addr: "".to_string(),
    }
}
