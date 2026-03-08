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

use common_base::{
    port::is_local_port_listening,
    role::{is_broker_node, is_engine_node},
};
use common_config::broker::broker_config;

pub fn healthy_ready_check() -> bool {
    let config = broker_config();

    // Admin and gRPC ports are always started by broker-server.
    if !is_local_port_listening(config.http_port) || !is_local_port_listening(config.grpc_port) {
        return false;
    }

    // MQTT listeners are only required on broker role nodes.
    if is_broker_node(&config.roles)
        && (!is_local_port_listening(config.mqtt_server.tcp_port)
            || !is_local_port_listening(config.mqtt_server.tls_port)
            || !is_local_port_listening(config.mqtt_server.websocket_port)
            || !is_local_port_listening(config.mqtt_server.websockets_port)
            || !is_local_port_listening(config.mqtt_server.quic_port))
    {
        return false;
    }

    // Storage engine TCP listener is only required on engine role nodes.
    if is_engine_node(&config.roles) && !is_local_port_listening(config.storage_runtime.tcp_port) {
        return false;
    }

    true
}
