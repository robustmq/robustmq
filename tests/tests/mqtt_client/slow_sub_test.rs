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

    use grpc_clients::mqtt::admin::call::{
        mqtt_broker_enable_slow_subscribe, mqtt_broker_list_slow_subscribe,
    };
    use grpc_clients::pool::ClientPool;
    use protocol::broker_mqtt::broker_mqtt_admin::{
        EnableSlowSubscribeRequest, ListSlowSubscribeRequest,
    };

    use crate::mqtt_protocol::common::broker_grpc_addr;

    #[tokio::test]
    async fn test_enable_slow_subscribe() {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let request = EnableSlowSubscribeRequest { is_enable: true };

        match mqtt_broker_enable_slow_subscribe(client_pool, &grpc_addr, request).await {
            Ok(data) => {
                println!("{:?}", data);
            }

            Err(e) => {
                eprintln!("Failed enable_slow_subscribe: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    #[tokio::test]
    async fn test_list_slow_subscribe() {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];
        let request = ListSlowSubscribeRequest {};

        match mqtt_broker_list_slow_subscribe(client_pool, &grpc_addr, request).await {
            Ok(data) => {
                println!("{:?}", data);
            }

            Err(e) => {
                eprintln!("Failed list slow subscribe: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}
