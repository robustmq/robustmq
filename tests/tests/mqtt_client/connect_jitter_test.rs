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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use grpc_clients::mqtt::admin::call::mqtt_broker_enable_connection_jitter;
    use grpc_clients::pool::ClientPool;
    use protocol::broker_mqtt::broker_mqtt_admin::{
        EnableConnectionJitterReply, EnableConnectionJitterRequest,
    };

    use crate::mqtt_protocol::common::broker_grpc_addr;

    #[tokio::test]
    async fn test_enable_connection_jitter() {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let request = EnableConnectionJitterRequest {
            is_enable: false,
            window_time: 0,
            max_client_connections: 0,
            ban_time: 0,
        };

        let reply = EnableConnectionJitterReply { is_enable: false };

        match mqtt_broker_enable_connection_jitter(&client_pool, &grpc_addr, request).await {
            Ok(data) => {
                assert_eq!(data, reply);
            }

            Err(e) => {
                eprintln!("Failed enable_slow_subscribe: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    #[tokio::test]
    async fn test_enable_connection_jitter_is_true() {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let request = EnableConnectionJitterRequest {
            is_enable: true,
            window_time: 1,
            max_client_connections: 3,
            ban_time: 5,
        };

        let reply = EnableConnectionJitterReply { is_enable: true };

        match mqtt_broker_enable_connection_jitter(&client_pool, &grpc_addr, request).await {
            Ok(data) => {
                assert_eq!(data, reply);
            }

            Err(e) => {
                eprintln!("Failed enable_slow_subscribe: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}
