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

use super::config::{Network, Shard, Storage, System, TcpThread};
use crate::common::Log;

pub fn default_network() -> Network {
    Network {
        local_ip: default_local_ip(),
        grpc_port: default_grpc_port(),
        tcp_port: default_network_tcp_port(),
        tcps_port: default_network_tcps_port(),
        tls_cert: "".to_string(),
        tls_key: "".to_string(),
    }
}

pub fn default_shard() -> Shard {
    Shard {
        enable_auto_create_shard: default_enable_auto_create_shard(),
        shard_replica_num: default_shard_replica_num(),
        max_segment_size: default_max_segment_size(),
    }
}

pub fn default_enable_auto_create_shard() -> bool {
    false
}

pub fn default_shard_replica_num() -> u32 {
    1
}

pub fn default_max_segment_size() -> u32 {
    1073741824
}

pub fn default_local_ip() -> String {
    "127.0.0.1".to_string()
}

pub fn default_grpc_port() -> u32 {
    2228
}
pub fn default_network_tcp_port() -> u32 {
    3110
}
pub fn default_network_tcps_port() -> u32 {
    3111
}

pub fn default_system() -> System {
    System {
        runtime_work_threads: 16,
    }
}

pub fn default_storage() -> Storage {
    Storage {
        data_path: vec!["".to_string()],
        rocksdb_max_open_files: None,
    }
}

pub fn default_tcp_thread() -> TcpThread {
    TcpThread {
        accept_thread_num: 1,
        handler_thread_num: 1,
        response_thread_num: 1,
        max_connection_num: 1000,
        request_queue_size: 2000,
        response_queue_size: 2000,
    }
}

pub fn default_log() -> Log {
    Log {
        log_path: "./logs".to_string(),
        log_config: "./config/log-config/journal-tracing.toml".to_string(),
    }
}
