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
    use common_base::error::common::CommonError;
    use grpc_clients::mqtt::admin::call::mqtt_broker_enable_flapping_detect;
    use grpc_clients::pool::ClientPool;

    use crate::mqtt_protocol::common::broker_grpc_addr;
    use protocol::broker_mqtt::broker_mqtt_admin::{
        EnableFlappingDetectReply, EnableFlappingDetectRequest,
    };
    use std::sync::Arc;

    async fn open_flapping_detect(
        request: EnableFlappingDetectRequest,
    ) -> Result<EnableFlappingDetectReply, CommonError> {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];
        mqtt_broker_enable_flapping_detect(&client_pool, &grpc_addr, request).await
    }

    #[tokio::test]
    async fn test_enable_flapping_detect() {
        // per
        let request = EnableFlappingDetectRequest {
            is_enable: true,
            window_time: 1,
            max_client_connections: 15,
            ban_time: 1,
        };

        // result
        let reply = EnableFlappingDetectReply { is_enable: true };

        // action
        match open_flapping_detect(request).await {
            Ok(data) => {
                assert_eq!(data, reply);
            }

            Err(e) => {
                eprintln!("Failed enable_slow_subscribe: {e:?}");
                std::process::exit(1);
            }
        }

        // per
        let request = EnableFlappingDetectRequest {
            is_enable: false,
            window_time: 1,
            max_client_connections: 15,
            ban_time: 1,
        };

        // result
        let reply = EnableFlappingDetectReply { is_enable: false };

        // action
        match open_flapping_detect(request).await {
            Ok(data) => {
                assert_eq!(data, reply);
            }

            Err(e) => {
                eprintln!("Failed enable_slow_subscribe: {e:?}");
                std::process::exit(1);
            }
        }
    }
}
